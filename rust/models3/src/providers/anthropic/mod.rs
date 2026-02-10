pub mod translate_error;
pub mod translate_request;
pub mod translate_response;
pub mod translate_stream;

use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use stencila_auth::{AuthCredential, StaticKey, api_key_header, bearer_header};

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

/// System prompt prefix injected when using OAuth Bearer authentication.
const OAUTH_SYSTEM_PREFIX: &str = "You are Claude Code, Anthropic's official CLI for Claude.";

/// Beta feature flags required for OAuth Bearer token authentication.
///
/// Matches the headers used by Claude Code / pi-mono:
/// <https://github.com/badlogic/pi-mono/blob/34878e7cc8074f42edff6c2cdcc9828aa9b6afde/packages/ai/src/providers/anthropic.ts#L487-L498>
const OAUTH_BETA_FEATURES: &[&str] = &["claude-code-20250219", "oauth-2025-04-20"];

/// Native Anthropic adapter using the Messages API (`/v1/messages`).
#[derive(Clone)]
pub struct AnthropicAdapter {
    http: HttpClient,
    auth: Arc<dyn AuthCredential>,
    /// When `true`, send `Authorization: Bearer` with the `oauth-2025-04-20`
    /// beta header instead of `x-api-key`. Set for OAuth credentials.
    use_bearer: bool,
}

impl std::fmt::Debug for AnthropicAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnthropicAdapter").finish_non_exhaustive()
    }
}

impl AnthropicAdapter {
    /// Create an adapter with an explicit API key and optional base URL.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn new(api_key: impl Into<String>, base_url: Option<String>) -> SdkResult<Self> {
        let auth: Arc<dyn AuthCredential> = Arc::new(StaticKey::new(api_key));
        Self::build(auth, base_url, false)
    }

    /// Create an adapter with an [`AuthCredential`] for OAuth token resolution.
    ///
    /// Uses `Authorization: Bearer` with the `oauth-2025-04-20` beta header
    /// instead of `x-api-key`.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn with_auth(auth: Arc<dyn AuthCredential>, base_url: Option<String>) -> SdkResult<Self> {
        Self::build(auth, base_url, true)
    }

    fn build(
        auth: Arc<dyn AuthCredential>,
        base_url: Option<String>,
        use_bearer: bool,
    ) -> SdkResult<Self> {
        let base = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let http = HttpClient::builder(base)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .build()?;
        Ok(Self {
            http,
            auth,
            use_bearer,
        })
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

    /// Get the auth header for a request.
    async fn auth_headers(&self) -> SdkResult<HeaderMap> {
        let token = self.auth.get_token().await?;
        Ok(if self.use_bearer {
            bearer_header(&token)?
        } else {
            api_key_header(&token)?
        })
    }

    /// System prompt prefix for OAuth requests, `None` for API key requests.
    fn system_prefix(&self) -> Option<&'static str> {
        if self.use_bearer {
            Some(OAUTH_SYSTEM_PREFIX)
        } else {
            None
        }
    }

    /// Merge auth headers with translated request headers.
    ///
    /// When using OAuth (`use_bearer`), appends the `oauth-2025-04-20` beta
    /// feature to the `anthropic-beta` header, merging with any existing
    /// beta features (e.g. `prompt-caching-2024-07-31`).
    fn merge_headers(&self, auth: HeaderMap, translated: &HeaderMap) -> HeaderMap {
        let mut headers = auth;
        for (k, v) in translated {
            headers.insert(k, v.clone());
        }
        if self.use_bearer {
            inject_oauth_headers(&mut headers);
        }
        headers
    }
}

/// Inject all OAuth-related headers required for Bearer token authentication.
///
/// Adds the `oauth-2025-04-20` and `claude-code-20250219` beta features
/// (merging with any existing `anthropic-beta` values), plus the
/// `anthropic-dangerous-direct-browser-access` header.
///
/// See: <https://github.com/badlogic/pi-mono/blob/34878e7cc8074f42edff6c2cdcc9828aa9b6afde/packages/ai/src/providers/anthropic.ts#L487-L498>
fn inject_oauth_headers(headers: &mut HeaderMap) {
    // Merge OAuth beta features into the anthropic-beta header
    let beta_name = HeaderName::from_static("anthropic-beta");

    let existing = headers
        .get(&beta_name)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();

    let existing_features: Vec<&str> = if existing.is_empty() {
        Vec::new()
    } else {
        existing.split(',').map(str::trim).collect()
    };

    let mut merged = existing.clone();
    for &feature in OAUTH_BETA_FEATURES {
        if !existing_features.contains(&feature) {
            if merged.is_empty() {
                merged = feature.to_string();
            } else {
                merged = format!("{merged},{feature}");
            }
        }
    }

    if let Ok(value) = HeaderValue::from_str(&merged) {
        headers.insert(beta_name, value);
    }

    // Required for OAuth token usage
    headers.insert(
        HeaderName::from_static("anthropic-dangerous-direct-browser-access"),
        HeaderValue::from_static("true"),
    );
    headers.insert(
        HeaderName::from_static("user-agent"),
        HeaderValue::from_static("claude-cli/2.1.37 (external, cli)"),
    );
    headers.insert(
        HeaderName::from_static("x-app"),
        HeaderValue::from_static("cli"),
    );
}

impl ProviderAdapter for AnthropicAdapter {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    fn complete(&self, request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        Box::pin(async move {
            let timeout = request.timeout;
            let translated =
                translate_request::translate_request(&request, false, self.system_prefix())?;

            let headers = self.merge_headers(self.auth_headers().await?, &translated.headers);

            let (raw_response, resp_headers) = self
                .http
                .post_json::<serde_json::Value, serde_json::Value>(
                    "/v1/messages",
                    &translated.body,
                    Some(&headers),
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
            let translated =
                translate_request::translate_request(&request, true, self.system_prefix())?;

            let headers = self.merge_headers(self.auth_headers().await?, &translated.headers);

            let (byte_stream, resp_headers) = self
                .http
                .post_stream(
                    "/v1/messages",
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

    fn supports_tool_choice(&self, choice: &ToolChoice) -> bool {
        matches!(
            choice,
            ToolChoice::Auto | ToolChoice::Required | ToolChoice::Tool(_) | ToolChoice::None
        )
    }

    fn list_models(&self) -> BoxFuture<'_, SdkResult<Vec<ModelInfo>>> {
        Box::pin(async move {
            let mut headers = self.auth_headers().await?;
            if self.use_bearer {
                inject_oauth_headers(&mut headers);
            }

            let (body, _headers): (serde_json::Value, _) = self
                .http
                .get_json::<serde_json::Value>("/v1/models", Some(&headers))
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
