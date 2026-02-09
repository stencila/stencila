pub mod translate_error;
pub mod translate_request;
pub mod translate_response;
pub mod translate_stream;

use crate::error::{SdkError, SdkResult};
use crate::http::client::HttpClient;
use crate::http::headers::parse_rate_limit_headers;
use crate::http::sse::parse_sse;
use crate::provider::{BoxFuture, BoxStream, ProviderAdapter};
use crate::types::request::Request;
use crate::types::response::Response;
use crate::types::stream_event::StreamEvent;
use crate::types::tool::ToolChoice;

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com";

/// Native Gemini adapter using the Gemini API.
///
/// Authentication is performed via an API key passed as a query parameter
/// on each request, following the Gemini API convention.
#[derive(Clone, Debug)]
pub struct GeminiAdapter {
    http: HttpClient,
    api_key: String,
}

impl GeminiAdapter {
    /// Create an adapter with an explicit API key and optional base URL.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> SdkResult<Self> {
        let base = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let api_key = api_key.into();
        let http = HttpClient::builder(base)
            .header("content-type", "application/json")
            .build()?;
        Ok(Self { http, api_key })
    }

    /// Create an adapter from environment variables.
    ///
    /// Reads `GEMINI_API_KEY` (required) and `GEMINI_BASE_URL` (optional).
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if `GEMINI_API_KEY` is not set.
    pub fn from_env() -> SdkResult<Self> {
        let api_key = std::env::var("GEMINI_API_KEY").map_err(|_| SdkError::Configuration {
            message: "GEMINI_API_KEY environment variable not set".into(),
        })?;
        let base_url = std::env::var("GEMINI_BASE_URL").ok();
        Self::new(api_key, base_url)
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

            let path = format!(
                "/v1beta/models/{}:generateContent?key={}",
                request.model, self.api_key
            );

            let (raw_response, headers) = self
                .http
                .post_json::<serde_json::Value, serde_json::Value>(
                    &path,
                    &translated_body,
                    None,
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
            let translated_body = translate_request::translate_request(&request)?;

            let path = format!(
                "/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
                request.model, self.api_key
            );

            let (byte_stream, headers) = self
                .http
                .post_stream(&path, &translated_body, None, timeout.as_ref())
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
}
