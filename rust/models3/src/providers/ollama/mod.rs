pub mod translate_error;
pub mod translate_request;
pub mod translate_response;
pub mod translate_stream;

use std::sync::Arc;

use reqwest::header::HeaderMap;
use stencila_auth::{AuthCredential, StaticKey, bearer_header};

use crate::catalog::ModelInfo;
use crate::error::SdkResult;
use crate::http::client::HttpClient;
use crate::http::headers::parse_rate_limit_headers;
use crate::http::sse::parse_sse;
use crate::provider::{BoxFuture, BoxStream, ProviderAdapter};
use crate::secret::get_secret;
use crate::types::request::Request;
use crate::types::response::Response;
use crate::types::stream_event::StreamEvent;

/// Default Ollama API base URL (the `/v1` prefix for OpenAI-compatible API).
const DEFAULT_BASE_URL: &str = "http://localhost:11434/v1";

/// Ollama adapter using the OpenAI-compatible Chat Completions API.
///
/// Ollama runs locally and does not require an API key by default.
/// An optional `OLLAMA_API_KEY` is supported for deployments behind
/// an auth proxy. An [`AuthCredential`] can be provided for OAuth.
#[derive(Clone)]
pub struct OllamaAdapter {
    http: HttpClient,
    auth: Option<Arc<dyn AuthCredential>>,
}

impl std::fmt::Debug for OllamaAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OllamaAdapter")
            .field("has_auth", &self.auth.is_some())
            .finish_non_exhaustive()
    }
}

impl OllamaAdapter {
    /// Create an adapter with an explicit base URL and optional API key.
    ///
    /// Unlike cloud providers, Ollama does not require authentication.
    /// Pass `api_key` only when Ollama is behind an auth proxy.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn new(base_url: impl Into<String>, api_key: Option<String>) -> SdkResult<Self> {
        let auth = api_key.map(|key| Arc::new(StaticKey::new(key)) as Arc<dyn AuthCredential>);
        Self::with_auth(base_url, auth)
    }

    /// Create an adapter with an [`AuthCredential`] for dynamic token resolution.
    ///
    /// Pass `None` for `auth` when no authentication is needed (the common local case).
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn with_auth(
        base_url: impl Into<String>,
        auth: Option<Arc<dyn AuthCredential>>,
    ) -> SdkResult<Self> {
        let base = base_url.into();
        let http = HttpClient::builder(base)
            .header("content-type", "application/json")
            .build()?;
        Ok(Self { http, auth })
    }

    /// Create an adapter from environment variables.
    ///
    /// Reads `OLLAMA_BASE_URL` or `OLLAMA_HOST` (appends `/v1` if needed)
    /// for the base URL, falling back to `http://localhost:11434/v1`.
    /// Optionally reads `OLLAMA_API_KEY` for auth-proxy deployments.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if HTTP client configuration is invalid.
    pub fn from_env() -> SdkResult<Self> {
        let base_url = Self::base_url_from_env_or_default();
        let api_key = get_secret("OLLAMA_API_KEY");
        Self::new(base_url, api_key)
    }

    /// Derive the base URL from environment variables, falling back to the default.
    ///
    /// Priority: `OLLAMA_BASE_URL` > `OLLAMA_HOST` (with scheme and `/v1` suffix) > default.
    ///
    /// This is public so that [`crate::client::Client::from_env_with_auth`] can
    /// resolve the Ollama URL without duplicating the logic.
    #[must_use]
    pub fn base_url_from_env_or_default() -> String {
        if let Ok(url) = std::env::var("OLLAMA_BASE_URL") {
            return url;
        }
        if let Ok(host) = std::env::var("OLLAMA_HOST") {
            let trimmed = host.trim_end_matches('/');
            // Ensure a scheme is present (Ollama's convention is host:port without scheme)
            let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                trimmed.to_string()
            } else {
                format!("http://{trimmed}")
            };
            return if with_scheme.ends_with("/v1") {
                with_scheme
            } else {
                format!("{with_scheme}/v1")
            };
        }
        DEFAULT_BASE_URL.to_string()
    }

    /// Check whether an Ollama instance is reachable at the given address.
    ///
    /// Attempts a TCP connection to `host:port` (e.g. `"localhost:11434"`).
    /// Returns `true` if the connection succeeds within a short timeout,
    /// `false` otherwise.
    #[must_use]
    pub fn is_available(addr: &str) -> bool {
        use std::net::{TcpStream, ToSocketAddrs};

        let Ok(mut addrs) = addr.to_socket_addrs() else {
            return false;
        };
        addrs.any(|a| TcpStream::connect_timeout(&a, std::time::Duration::from_secs(1)).is_ok())
    }

    /// Get the auth header for a request, if authentication is configured.
    async fn auth_headers(&self) -> SdkResult<Option<HeaderMap>> {
        match &self.auth {
            Some(auth) => {
                let token = auth.get_token().await?;
                Ok(Some(bearer_header(&token)?))
            }
            None => Ok(None),
        }
    }
}

impl ProviderAdapter for OllamaAdapter {
    fn name(&self) -> &'static str {
        "ollama"
    }

    fn complete(&self, request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        Box::pin(async move {
            let timeout = request.timeout;
            let translated = translate_request::translate_request(&request, false)?;

            let auth_headers = self.auth_headers().await?;
            let headers = match auth_headers {
                Some(mut auth) => {
                    for (k, v) in &translated.headers {
                        auth.insert(k, v.clone());
                    }
                    auth
                }
                None => translated.headers.clone(),
            };

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

            let auth_headers = self.auth_headers().await?;
            let headers = match auth_headers {
                Some(mut auth) => {
                    for (k, v) in &translated.headers {
                        auth.insert(k, v.clone());
                    }
                    auth
                }
                None => translated.headers.clone(),
            };

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
                .get_json::<serde_json::Value>("/models", auth_headers.as_ref())
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
                                provider: "ollama".into(),
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
