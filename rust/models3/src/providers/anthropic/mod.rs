pub mod translate_error;
pub mod translate_request;
pub mod translate_response;
pub mod translate_stream;

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

/// Default Anthropic API base URL.
const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";

/// Anthropic API version header value.
const API_VERSION: &str = "2023-06-01";

/// Native Anthropic adapter using the Messages API (`/v1/messages`).
#[derive(Clone, Debug)]
pub struct AnthropicAdapter {
    http: HttpClient,
}

impl AnthropicAdapter {
    /// Create an adapter with an explicit API key and optional base URL.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> SdkResult<Self> {
        let base = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let http = HttpClient::builder(base)
            .header("x-api-key", api_key.into())
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .build()?;
        Ok(Self { http })
    }

    /// Create an adapter from environment variables.
    ///
    /// Reads `ANTHROPIC_API_KEY` (required) and `ANTHROPIC_BASE_URL` (optional).
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if `ANTHROPIC_API_KEY` is not set.
    pub fn from_env() -> SdkResult<Self> {
        let api_key = get_secret("ANTHROPIC_API_KEY").ok_or(SdkError::Configuration {
            message: format!(
                "ANTHROPIC_API_KEY not found in {}",
                secret_source_description()
            ),
        })?;
        let base_url = std::env::var("ANTHROPIC_BASE_URL").ok();
        Self::new(api_key, base_url)
    }
}

impl ProviderAdapter for AnthropicAdapter {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    fn complete(&self, request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        Box::pin(async move {
            let timeout = request.timeout;
            let translated = translate_request::translate_request(&request, false)?;

            let (raw_response, headers) = self
                .http
                .post_json::<serde_json::Value, serde_json::Value>(
                    "/v1/messages",
                    &translated.body,
                    Some(&translated.headers),
                    timeout.as_ref(),
                )
                .await
                .map_err(translate_error::translate_error)?;

            let rate_limit = parse_rate_limit_headers(&headers);
            translate_response::translate_response(raw_response, rate_limit)
        })
    }

    fn stream(
        &self,
        request: Request,
    ) -> BoxFuture<'_, SdkResult<BoxStream<'_, SdkResult<StreamEvent>>>> {
        Box::pin(async move {
            let timeout = request.timeout;
            let translated = translate_request::translate_request(&request, true)?;

            let (byte_stream, headers) = self
                .http
                .post_stream(
                    "/v1/messages",
                    &translated.body,
                    Some(&translated.headers),
                    timeout.as_ref(),
                )
                .await
                .map_err(translate_error::translate_error)?;

            let rate_limit = parse_rate_limit_headers(&headers);
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
            let (body, _headers): (serde_json::Value, _) = self
                .http
                .get_json::<serde_json::Value>("/v1/models", None)
                .await?;

            let models = body
                .get("data")
                .and_then(|d| d.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| {
                            let id = m.get("id")?.as_str()?.to_string();
                            let display_name = m
                                .get("display_name")
                                .and_then(|n| n.as_str())
                                .unwrap_or(&id)
                                .to_string();
                            Some(ModelInfo {
                                id,
                                provider: "anthropic".into(),
                                display_name,
                                context_window: 0,
                                max_output: None,
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
