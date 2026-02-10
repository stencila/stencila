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

/// Native OpenAI adapter using the Responses API (`/v1/responses`).
#[derive(Clone)]
pub struct OpenAIAdapter {
    http: HttpClient,
    auth: Arc<dyn AuthCredential>,
}

impl std::fmt::Debug for OpenAIAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAIAdapter").finish_non_exhaustive()
    }
}

impl OpenAIAdapter {
    /// Create an adapter with the default OpenAI API base URL.
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
        let auth: Arc<dyn AuthCredential> = Arc::new(StaticKey::new(api_key));
        Self::with_auth(auth, base_url, organization, project)
    }

    /// Create an adapter with an [`AuthCredential`] for dynamic token resolution.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn with_auth(
        auth: Arc<dyn AuthCredential>,
        base_url: impl Into<String>,
        organization: Option<impl Into<String>>,
        project: Option<impl Into<String>>,
    ) -> SdkResult<Self> {
        Self::with_auth_and_account(auth, base_url, organization, project, None::<String>)
    }

    /// Create an adapter with an [`AuthCredential`] and optional `ChatGPT` account ID.
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
        organization: Option<impl Into<String>>,
        project: Option<impl Into<String>>,
        chatgpt_account_id: Option<impl Into<String>>,
    ) -> SdkResult<Self> {
        let mut builder = HttpClient::builder(base_url).header("content-type", "application/json");

        if let Some(organization) = organization {
            builder = builder.header("openai-organization", organization.into());
        }
        if let Some(project) = project {
            builder = builder.header("openai-project", project.into());
        }
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

impl ProviderAdapter for OpenAIAdapter {
    fn name(&self) -> &'static str {
        "openai"
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
                    "/responses",
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
                    "/responses",
                    &translated.body,
                    Some(&headers),
                    timeout.as_ref(),
                )
                .await
                .map_err(translate_error::translate_error)?;

            let rate_limit = parse_rate_limit_headers(&resp_headers);
            let sse_stream = parse_sse(byte_stream);
            let unified_stream = translate_stream::translate_sse_stream(sse_stream, rate_limit);
            Ok(unified_stream)
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
                                provider: "openai".into(),
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
