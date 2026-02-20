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

/// Default DeepSeek API base URL.
const DEFAULT_BASE_URL: &str = "https://api.deepseek.com/v1";

/// DeepSeek adapter using the OpenAI-compatible Chat Completions API.
#[derive(Clone)]
pub struct DeepSeekAdapter {
    http: HttpClient,
    auth: Arc<dyn AuthCredential>,
}

impl std::fmt::Debug for DeepSeekAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeepSeekAdapter").finish_non_exhaustive()
    }
}

impl DeepSeekAdapter {
    /// Create an adapter with an explicit API key and optional base URL.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> SdkResult<Self> {
        let auth: Arc<dyn AuthCredential> = Arc::new(StaticKey::new(api_key));
        Self::with_auth(auth, base_url)
    }

    /// Create an adapter with an [`AuthCredential`] for dynamic token resolution.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn with_auth(auth: Arc<dyn AuthCredential>, base_url: Option<String>) -> SdkResult<Self> {
        let base = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let http = HttpClient::builder(base)
            .header("content-type", "application/json")
            .build()?;
        Ok(Self { http, auth })
    }

    /// Create an adapter from environment variables.
    ///
    /// Reads `DEEPSEEK_API_KEY` (required) and `DEEPSEEK_BASE_URL` (optional).
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if `DEEPSEEK_API_KEY` is not set.
    pub fn from_env() -> SdkResult<Self> {
        let api_key = get_secret("DEEPSEEK_API_KEY").ok_or(SdkError::Configuration {
            message: format!(
                "DEEPSEEK_API_KEY not found in {}",
                secret_source_description()
            ),
        })?;
        let base_url = std::env::var("DEEPSEEK_BASE_URL").ok();
        Self::new(api_key, base_url)
    }

    /// Get the auth header for a request.
    async fn auth_headers(&self) -> SdkResult<HeaderMap> {
        let token = self.auth.get_token().await?;
        Ok(bearer_header(&token)?)
    }
}

impl ProviderAdapter for DeepSeekAdapter {
    fn name(&self) -> &'static str {
        "deepseek"
    }

    fn complete(&self, request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        Box::pin(async move {
            let timeout = request.timeout;
            let translated = translate_request::translate_request(&request, false)?;

            let mut headers = self.auth_headers().await?;
            for (k, v) in &translated.headers {
                headers.insert(k, v.clone());
            }

            let (raw_response, resp_headers) = self
                .http
                .post_json::<serde_json::Value, serde_json::Value>(
                    "/chat/completions",
                    &translated.body,
                    Some(&headers),
                    timeout.as_ref(),
                )
                .await
                .map_err(translate_error::translate_error)?;

            translate_response::translate_response(raw_response, Some(&resp_headers))
        })
    }

    fn stream(
        &self,
        request: Request,
    ) -> BoxFuture<'_, SdkResult<BoxStream<'_, SdkResult<StreamEvent>>>> {
        Box::pin(async move {
            let timeout = request.timeout;
            let translated = translate_request::translate_request(&request, true)?;

            let mut headers = self.auth_headers().await?;
            for (k, v) in &translated.headers {
                headers.insert(k, v.clone());
            }

            let (byte_stream, resp_headers) = self
                .http
                .post_stream(
                    "/chat/completions",
                    &translated.body,
                    Some(&headers),
                    timeout.as_ref(),
                )
                .await
                .map_err(translate_error::translate_error)?;

            let rate_limit = parse_rate_limit_headers(&resp_headers);
            let sse_stream = parse_sse(byte_stream);
            Ok(translate_stream::translate_sse_stream(
                sse_stream, rate_limit,
            ))
        })
    }

    fn supports_tool_choice(&self, _choice: &crate::types::tool::ToolChoice) -> bool {
        true
    }

    fn list_models(&self) -> BoxFuture<'_, SdkResult<Vec<ModelInfo>>> {
        Box::pin(async move {
            let auth_headers = self.auth_headers().await?;

            let (body, _headers): (serde_json::Value, _) = self
                .http
                .get_json::<serde_json::Value>("/models", Some(&auth_headers))
                .await?;

            let models = body
                .get("data")
                .and_then(|d| d.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| {
                            let id = m.get("id")?.as_str()?.to_string();

                            Some(ModelInfo {
                                id: id.clone(),
                                provider: "deepseek".into(),
                                display_name: id,
                                context_window: 0,
                                max_output: None,
                                supports_tools: false,
                                supports_vision: false,
                                supports_reasoning: false,
                                input_cost_per_million: None,
                                output_cost_per_million: None,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            Ok(models)
        })
    }
}
