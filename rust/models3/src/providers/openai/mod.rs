pub mod translate_error;
pub mod translate_request;
pub mod translate_response;
pub mod translate_stream;

use crate::error::SdkResult;
use crate::http::client::HttpClient;
use crate::http::headers::parse_rate_limit_headers;
use crate::http::sse::parse_sse;
use crate::provider::{BoxFuture, BoxStream, ProviderAdapter};
use crate::types::request::Request;
use crate::types::response::Response;
use crate::types::stream_event::StreamEvent;

/// Native `OpenAI` adapter using the Responses API (`/v1/responses`).
#[derive(Clone, Debug)]
pub struct OpenAIAdapter {
    http: HttpClient,
}

impl OpenAIAdapter {
    /// Create an adapter with the default `OpenAI` API base URL.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn new(api_key: impl Into<String>) -> SdkResult<Self> {
        Self::with_config(
            api_key,
            "https://api.openai.com/v1",
            None::<String>,
            None::<String>,
        )
    }

    /// Create an adapter with explicit base URL, org, and project settings.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn with_config(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        organization: Option<impl Into<String>>,
        project: Option<impl Into<String>>,
    ) -> SdkResult<Self> {
        let mut builder = HttpClient::builder(base_url)
            .header("authorization", format!("Bearer {}", api_key.into()))
            .header("content-type", "application/json");

        if let Some(organization) = organization {
            builder = builder.header("openai-organization", organization.into());
        }
        if let Some(project) = project {
            builder = builder.header("openai-project", project.into());
        }

        Ok(Self {
            http: builder.build()?,
        })
    }
}

impl ProviderAdapter for OpenAIAdapter {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn complete(&self, request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        Box::pin(async move {
            let timeout = request.timeout;
            let translated = translate_request::translate_request(&request, false)?;

            let (raw_response, headers) = self
                .http
                .post_json::<serde_json::Value, serde_json::Value>(
                    "/responses",
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
                    "/responses",
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

    fn supports_tool_choice(&self, _choice: &crate::types::tool::ToolChoice) -> bool {
        true
    }
}
