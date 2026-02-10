pub mod translate_error;
pub mod translate_request;
pub mod translate_response;
pub mod translate_stream;

use std::sync::Arc;

use reqwest::header::HeaderMap;

use crate::auth::{AuthCredential, StaticKey, bearer_header};
use crate::catalog::ModelInfo;
use crate::error::SdkResult;
use crate::http::client::HttpClient;
use crate::http::headers::parse_rate_limit_headers;
use crate::http::sse::parse_sse;
use crate::provider::{BoxFuture, BoxStream, ProviderAdapter};
use crate::providers::common::openai_shared::is_excluded_openai_model;
use crate::types::request::Request;
use crate::types::response::Response;
use crate::types::stream_event::StreamEvent;

/// OpenAI-compatible adapter for Chat Completions endpoints.
///
/// This is intended for third-party services that implement `/v1/chat/completions`.
#[derive(Clone)]
pub struct OpenAIChatCompletionsAdapter {
    http: HttpClient,
    auth: Arc<dyn AuthCredential>,
}

impl std::fmt::Debug for OpenAIChatCompletionsAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAIChatCompletionsAdapter")
            .finish_non_exhaustive()
    }
}

impl OpenAIChatCompletionsAdapter {
    /// Create an adapter with the default OpenAI base URL.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn new(api_key: impl Into<String>) -> SdkResult<Self> {
        Self::with_base_url(api_key, "https://api.openai.com/v1")
    }

    /// Create an adapter with a custom OpenAI-compatible base URL.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn with_base_url(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
    ) -> SdkResult<Self> {
        let auth: Arc<dyn AuthCredential> = Arc::new(StaticKey::new(api_key));
        Self::with_auth(auth, base_url)
    }

    /// Create an adapter with an [`AuthCredential`] and custom base URL.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn with_auth(
        auth: Arc<dyn AuthCredential>,
        base_url: impl Into<String>,
    ) -> SdkResult<Self> {
        Self::with_auth_and_account(auth, base_url, None::<String>)
    }

    /// Create an adapter with an [`AuthCredential`], custom base URL, and
    /// optional `ChatGPT` account ID.
    ///
    /// The `chatgpt_account_id` is sent as the `ChatGPT-Account-Id` header,
    /// required when authenticating via the OpenAI OAuth (Codex) flow.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn with_auth_and_account(
        auth: Arc<dyn AuthCredential>,
        base_url: impl Into<String>,
        chatgpt_account_id: Option<impl Into<String>>,
    ) -> SdkResult<Self> {
        let mut builder = HttpClient::builder(base_url).header("content-type", "application/json");

        if let Some(account_id) = chatgpt_account_id {
            builder = builder.header("chatgpt-account-id", account_id.into());
        }

        Ok(Self {
            http: builder.build()?,
            auth,
        })
    }

    /// Get the auth header for a request.
    async fn auth_headers(&self) -> SdkResult<HeaderMap> {
        let token = self.auth.get_token().await?;
        bearer_header(&token)
    }
}

impl ProviderAdapter for OpenAIChatCompletionsAdapter {
    fn name(&self) -> &'static str {
        "openai_chat_completions"
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
                            if is_excluded_openai_model(&id) {
                                return None;
                            }
                            Some(ModelInfo {
                                id: id.clone(),
                                provider: "openai_chat_completions".into(),
                                display_name: id,
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
