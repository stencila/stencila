pub mod translate_error;
pub mod translate_request;
pub mod translate_response;
pub mod translate_stream;

use std::sync::Arc;

use reqwest::header::HeaderMap;
use stencila_auth::{AuthCredential, StaticKey, bearer_header};

use crate::catalog::ModelInfo;
use crate::error::{SdkError, SdkResult};
use crate::http::client::HttpClient;
use crate::http::headers::parse_rate_limit_headers;
use crate::http::sse::parse_sse;
use crate::provider::{BoxFuture, BoxStream, ProviderAdapter};
use crate::secret::{get_secret, secret_source_description};
use crate::types::request::Request;
use crate::types::response::Response;
use crate::types::stream_event::StreamEvent;
use crate::types::tool::ToolChoice;

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com";

/// How the Gemini adapter transmits its authentication credential.
#[derive(Debug, Clone)]
enum GeminiAuthTransport {
    /// API key appended as `?key={token}` query parameter (standard Gemini API keys).
    QueryParam,
    /// OAuth token sent as `Authorization: Bearer {token}` header.
    /// Used for Google Cloud OAuth so tokens are not leaked in URLs.
    BearerHeader,
}

/// Native Gemini adapter using the Gemini API.
///
/// Supports two authentication transports:
/// - **Query parameter** (default for API keys): `?key={token}`
/// - **Bearer header** (for OAuth tokens): `Authorization: Bearer {token}`
#[derive(Clone)]
pub struct GeminiAdapter {
    http: HttpClient,
    auth: Arc<dyn AuthCredential>,
    transport: GeminiAuthTransport,
}

impl std::fmt::Debug for GeminiAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeminiAdapter")
            .field("transport", &self.transport)
            .finish_non_exhaustive()
    }
}

impl GeminiAdapter {
    /// Create an adapter with an explicit API key and optional base URL.
    ///
    /// Uses query-parameter authentication (the standard Gemini API key transport).
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> SdkResult<Self> {
        let base = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let auth: Arc<dyn AuthCredential> = Arc::new(StaticKey::new(api_key));
        let http = HttpClient::builder(base)
            .header("content-type", "application/json")
            .build()?;
        Ok(Self {
            http,
            auth,
            transport: GeminiAuthTransport::QueryParam,
        })
    }

    /// Create an adapter with an [`AuthCredential`] using Bearer header transport.
    ///
    /// This is the correct transport for OAuth tokens (Google Cloud OAuth),
    /// preventing token leakage in URLs and server logs.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn with_auth(auth: Arc<dyn AuthCredential>, base_url: Option<String>) -> SdkResult<Self> {
        let base = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let http = HttpClient::builder(base)
            .header("content-type", "application/json")
            .build()?;
        Ok(Self {
            http,
            auth,
            transport: GeminiAuthTransport::BearerHeader,
        })
    }

    /// Create an adapter from environment variables.
    ///
    /// Reads `GEMINI_API_KEY` (required) and `GEMINI_BASE_URL` (optional).
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if `GEMINI_API_KEY` is not set.
    pub fn from_env() -> SdkResult<Self> {
        let api_key = get_secret("GEMINI_API_KEY").ok_or(SdkError::Configuration {
            message: format!(
                "GEMINI_API_KEY not found in {}",
                secret_source_description()
            ),
        })?;
        let base_url = std::env::var("GEMINI_BASE_URL").ok();
        Self::new(api_key, base_url)
    }

    /// Build the request path for a generate call, including auth if using query params.
    async fn generate_path(&self, model: &str, stream: bool) -> SdkResult<String> {
        let method = if stream {
            "streamGenerateContent"
        } else {
            "generateContent"
        };
        let mut path = if stream {
            format!("/v1beta/models/{model}:{method}?alt=sse")
        } else {
            format!("/v1beta/models/{model}:{method}")
        };

        if matches!(self.transport, GeminiAuthTransport::QueryParam) {
            let token = self.auth.get_token().await?;
            let separator = if path.contains('?') { '&' } else { '?' };
            path = format!("{path}{separator}key={token}");
        }

        Ok(path)
    }

    /// Build auth headers for Bearer transport, or empty headers for query param.
    async fn auth_headers(&self) -> SdkResult<Option<HeaderMap>> {
        match self.transport {
            GeminiAuthTransport::BearerHeader => {
                let token = self.auth.get_token().await?;
                Ok(Some(bearer_header(&token)?))
            }
            GeminiAuthTransport::QueryParam => Ok(None),
        }
    }

    /// Build the path for listing models, including auth if using query params.
    async fn list_models_path(&self) -> SdkResult<String> {
        let mut path = "/v1beta/models".to_string();
        if matches!(self.transport, GeminiAuthTransport::QueryParam) {
            let token = self.auth.get_token().await?;
            path = format!("{path}?key={token}");
        }
        Ok(path)
    }
}

impl ProviderAdapter for GeminiAdapter {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn complete(&self, request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        Box::pin(async move {
            let timeout = request.timeout;
            let translated_body = translate_request::translate_request(&request)?;
            let path = self.generate_path(&request.model, false).await?;
            let headers = self.auth_headers().await?;

            let (raw_response, resp_headers) = self
                .http
                .post_json::<serde_json::Value, serde_json::Value>(
                    &path,
                    &translated_body,
                    headers.as_ref(),
                    timeout.as_ref(),
                )
                .await
                .map_err(translate_error::translate_error)?;

            let rate_limit = parse_rate_limit_headers(&resp_headers);
            translate_response::translate_response(raw_response, rate_limit)
        })
    }

    fn stream(
        &self,
        request: Request,
    ) -> BoxFuture<'_, SdkResult<BoxStream<'_, SdkResult<StreamEvent>>>> {
        Box::pin(async move {
            let timeout = request.timeout;
            let translated_body = translate_request::translate_request(&request)?;
            let path = self.generate_path(&request.model, true).await?;
            let headers = self.auth_headers().await?;

            let (byte_stream, resp_headers) = self
                .http
                .post_stream(&path, &translated_body, headers.as_ref(), timeout.as_ref())
                .await
                .map_err(translate_error::translate_error)?;

            let rate_limit = parse_rate_limit_headers(&resp_headers);
            let sse_stream = parse_sse(byte_stream);
            let unified_stream = translate_stream::translate_sse_stream(sse_stream, rate_limit);
            Ok(unified_stream)
        })
    }

    fn supports_tool_choice(&self, choice: &ToolChoice) -> bool {
        matches!(
            choice,
            ToolChoice::Auto | ToolChoice::Required | ToolChoice::Tool(_) | ToolChoice::None
        )
    }

    fn list_models(&self) -> BoxFuture<'_, SdkResult<Vec<ModelInfo>>> {
        Box::pin(async move {
            let path = self.list_models_path().await?;
            let headers = self.auth_headers().await?;

            let (body, _headers): (serde_json::Value, _) = self
                .http
                .get_json::<serde_json::Value>(&path, headers.as_ref())
                .await?;

            let models = body
                .get("models")
                .and_then(|d| d.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| {
                            // Only include models that support generateContent
                            let supports_generate = m
                                .get("supportedGenerationMethods")
                                .and_then(|v| v.as_array())
                                .is_some_and(|methods| {
                                    methods
                                        .iter()
                                        .any(|v| v.as_str() == Some("generateContent"))
                                });
                            if !supports_generate {
                                return None;
                            }

                            // Gemini returns "models/gemini-2.0-flash", strip prefix
                            let raw_name = m.get("name")?.as_str()?;
                            let id = raw_name
                                .strip_prefix("models/")
                                .unwrap_or(raw_name)
                                .to_string();
                            let display_name = m
                                .get("displayName")
                                .and_then(|n| n.as_str())
                                .unwrap_or(&id)
                                .to_string();
                            let context_window = m
                                .get("inputTokenLimit")
                                .and_then(serde_json::Value::as_u64)
                                .unwrap_or(0);
                            let max_output = m
                                .get("outputTokenLimit")
                                .and_then(serde_json::Value::as_u64);
                            Some(ModelInfo {
                                id,
                                provider: "gemini".into(),
                                display_name,
                                context_window,
                                max_output,
                                supports_tools: false,
                                supports_vision: false,
                                supports_reasoning: false,
                                input_cost_per_million: None,
                                output_cost_per_million: None,
                                aliases: vec![],
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            Ok(models)
        })
    }
}
