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

/// Default Mistral API base URL.
const DEFAULT_BASE_URL: &str = "https://api.mistral.ai/v1";

/// Mistral AI adapter using the Chat Completions API.
///
/// Mistral uses the same wire format as OpenAI Chat Completions, with one
/// notable difference: Mistral rejects `null` values in request bodies.
#[derive(Clone, Debug)]
pub struct MistralAdapter {
    http: HttpClient,
}

impl MistralAdapter {
    /// Create an adapter with an explicit API key and optional base URL.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> SdkResult<Self> {
        let base = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let http = HttpClient::builder(base)
            .header("authorization", format!("Bearer {}", api_key.into()))
            .header("content-type", "application/json")
            .build()?;
        Ok(Self { http })
    }

    /// Create an adapter from environment variables.
    ///
    /// Reads `MISTRAL_API_KEY` (required) and `MISTRAL_BASE_URL` (optional).
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if `MISTRAL_API_KEY` is not set.
    pub fn from_env() -> SdkResult<Self> {
        let api_key = get_secret("MISTRAL_API_KEY").ok_or(SdkError::Configuration {
            message: format!(
                "MISTRAL_API_KEY not found in {}",
                secret_source_description()
            ),
        })?;
        let base_url = std::env::var("MISTRAL_BASE_URL").ok();
        Self::new(api_key, base_url)
    }
}

impl ProviderAdapter for MistralAdapter {
    fn name(&self) -> &'static str {
        "mistral"
    }

    fn complete(&self, request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        Box::pin(async move {
            let timeout = request.timeout;
            let translated = translate_request::translate_request(&request, false)?;

            let (raw_response, headers) = self
                .http
                .post_json::<serde_json::Value, serde_json::Value>(
                    "/chat/completions",
                    &translated.body,
                    Some(&translated.headers),
                    timeout.as_ref(),
                )
                .await
                .map_err(translate_error::translate_error)?;

            translate_response::translate_response(raw_response, Some(&headers))
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
                    "/chat/completions",
                    &translated.body,
                    Some(&translated.headers),
                    timeout.as_ref(),
                )
                .await
                .map_err(translate_error::translate_error)?;

            let rate_limit = parse_rate_limit_headers(&headers);
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
            let (body, _headers): (serde_json::Value, _) = self
                .http
                .get_json::<serde_json::Value>("/models", None)
                .await?;

            let models = body
                .get("data")
                .and_then(|d| d.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| {
                            let id = m.get("id")?.as_str()?.to_string();

                            // Extract capabilities from Mistral's richer response
                            let capabilities = m.get("capabilities");

                            // Only include models that support chat completions
                            let supports_chat = capabilities
                                .and_then(|c| c.get("completion_chat"))
                                .and_then(serde_json::Value::as_bool)
                                .unwrap_or(false);
                            if !supports_chat {
                                return None;
                            }
                            let supports_tools = capabilities
                                .and_then(|c| c.get("function_calling"))
                                .and_then(serde_json::Value::as_bool)
                                .unwrap_or(false);
                            let supports_vision = capabilities
                                .and_then(|c| c.get("vision"))
                                .and_then(serde_json::Value::as_bool)
                                .unwrap_or(false);

                            let context_window = m
                                .get("max_context_length")
                                .and_then(serde_json::Value::as_u64)
                                .unwrap_or(0);

                            Some(ModelInfo {
                                id: id.clone(),
                                provider: "mistral".into(),
                                display_name: id,
                                context_window,
                                max_output: None,
                                supports_tools,
                                supports_vision,
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
