//! HTTP client wrapper for provider API calls.
//!
//! Wraps [`reqwest::Client`] with LLM-provider-friendly defaults:
//! timeouts, base URL, default headers, and convenience methods for
//! JSON POST and streaming POST (the two request patterns used by
//! every provider).

use std::collections::HashMap;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::error::{SdkError, SdkResult};
use crate::types::timeout::Timeout;

/// Default connect timeout in seconds.
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;

/// Default request timeout in seconds (spec §4.8: 120s).
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 120;

/// An HTTP client configured for LLM provider API calls.
#[derive(Debug, Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
    base_url: String,
    default_headers: HeaderMap,
}

impl HttpClient {
    /// Create a new [`HttpClientBuilder`].
    #[must_use]
    pub fn builder(base_url: impl Into<String>) -> HttpClientBuilder {
        HttpClientBuilder {
            base_url: base_url.into(),
            headers: HashMap::new(),
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
        }
    }

    /// The base URL for this client.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Send a GET request and return the parsed JSON response body.
    ///
    /// Used for model listing endpoints that return JSON.
    ///
    /// # Errors
    ///
    /// Returns an appropriate `SdkError` variant for network failures,
    /// timeouts, and non-success HTTP status codes.
    pub async fn get_json<R: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        headers: Option<&HeaderMap>,
    ) -> SdkResult<(R, HeaderMap)> {
        let url = format!("{}{path}", self.base_url);

        let mut request = self.inner.get(&url).headers(self.default_headers.clone());

        if let Some(h) = headers {
            request = request.headers(h.clone());
        }

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                SdkError::RequestTimeout {
                    message: e.to_string(),
                }
            } else if e.is_connect() {
                SdkError::Network {
                    message: format!("connection failed: {e}"),
                }
            } else {
                SdkError::Network {
                    message: e.to_string(),
                }
            }
        })?;

        let status = response.status();
        let response_headers = response.headers().clone();

        if !status.is_success() {
            let raw_body = response.text().await.unwrap_or_else(|_| String::new());
            let raw_json: Option<serde_json::Value> = serde_json::from_str(&raw_body).ok();
            let message = extract_error_message(raw_json.as_ref(), &raw_body);
            return Err(SdkError::from_status_code(
                status.as_u16(),
                message,
                None,
                None,
                crate::http::headers::parse_retry_after(&response_headers),
                raw_json,
            ));
        }

        let body: R = response.json().await.map_err(|e| SdkError::Network {
            message: format!("failed to parse response body: {e}"),
        })?;

        Ok((body, response_headers))
    }

    /// Send a JSON POST request and return the parsed response body.
    ///
    /// The `path` is appended to the base URL. Additional `headers` are
    /// merged with the client's defaults (per-request headers win).
    ///
    /// # Timeout handling
    ///
    /// Only `Timeout.request` is applied per-request (overrides the client
    /// default). `Timeout.connect` is a client-level setting applied at
    /// construction via [`HttpClientBuilder::connect_timeout`]. `Timeout.stream_idle`
    /// requires a stream wrapper and is enforced at a higher layer.
    ///
    /// # Errors
    ///
    /// Returns an appropriate `SdkError` variant for network failures,
    /// timeouts, and non-success HTTP status codes.
    pub async fn post_json<B: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        headers: Option<&HeaderMap>,
        timeout: Option<&Timeout>,
    ) -> SdkResult<(R, HeaderMap)> {
        let url = format!("{}{path}", self.base_url);

        let mut request = self.inner.post(&url).headers(self.default_headers.clone());

        if let Some(h) = headers {
            request = request.headers(h.clone());
        }

        if let Some(t) = timeout
            && let Some(secs) = t.request
            && secs.is_finite()
            && secs > 0.0
        {
            request = request.timeout(Duration::from_secs_f64(secs));
        }

        request = request.json(body);

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                SdkError::RequestTimeout {
                    message: e.to_string(),
                }
            } else if e.is_connect() {
                SdkError::Network {
                    message: format!("connection failed: {e}"),
                }
            } else {
                SdkError::Network {
                    message: e.to_string(),
                }
            }
        })?;

        let status = response.status();
        let response_headers = response.headers().clone();

        if !status.is_success() {
            let raw_body = response.text().await.unwrap_or_else(|_| String::new());
            let raw_json: Option<serde_json::Value> = serde_json::from_str(&raw_body).ok();
            let message = extract_error_message(raw_json.as_ref(), &raw_body);
            // Note: only status-code mapping is applied here. Message-based
            // classification for ambiguous cases (spec §7.6 step 4) is
            // provider-specific and applied in each adapter's translate_error
            // using SdkError::classify_from_message().
            return Err(SdkError::from_status_code(
                status.as_u16(),
                message,
                None,
                None,
                crate::http::headers::parse_retry_after(&response_headers),
                raw_json,
            ));
        }

        let body: R = response.json().await.map_err(|e| SdkError::Network {
            message: format!("failed to parse response body: {e}"),
        })?;

        Ok((body, response_headers))
    }

    /// Send a POST request and return a byte stream for SSE consumption.
    ///
    /// Returns the response headers alongside the byte stream so callers
    /// can extract rate-limit info.
    ///
    /// # Timeout handling
    ///
    /// Only `Timeout.request` is applied per-request. `Timeout.stream_idle`
    /// (inter-chunk idle timeout) must be enforced by wrapping the returned
    /// stream at a higher layer.
    ///
    /// # Errors
    ///
    /// Returns an appropriate `SdkError` variant for network failures,
    /// timeouts, and non-success HTTP status codes.
    pub async fn post_stream<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
        headers: Option<&HeaderMap>,
        timeout: Option<&Timeout>,
    ) -> SdkResult<(
        std::pin::Pin<Box<dyn futures::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
        HeaderMap,
    )> {
        let url = format!("{}{path}", self.base_url);

        let mut request = self.inner.post(&url).headers(self.default_headers.clone());

        if let Some(h) = headers {
            request = request.headers(h.clone());
        }

        if let Some(t) = timeout
            && let Some(secs) = t.request
            && secs.is_finite()
            && secs > 0.0
        {
            request = request.timeout(Duration::from_secs_f64(secs));
        }

        request = request.json(body);

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                SdkError::RequestTimeout {
                    message: e.to_string(),
                }
            } else if e.is_connect() {
                SdkError::Network {
                    message: format!("connection failed: {e}"),
                }
            } else {
                SdkError::Network {
                    message: e.to_string(),
                }
            }
        })?;

        let status = response.status();
        let response_headers = response.headers().clone();

        if !status.is_success() {
            let raw_body = response.text().await.unwrap_or_else(|_| String::new());
            let raw_json: Option<serde_json::Value> = serde_json::from_str(&raw_body).ok();
            let message = extract_error_message(raw_json.as_ref(), &raw_body);
            // See note in post_json: message-based classification is deferred
            // to provider adapters.
            return Err(SdkError::from_status_code(
                status.as_u16(),
                message,
                None,
                None,
                crate::http::headers::parse_retry_after(&response_headers),
                raw_json,
            ));
        }

        let stream = Box::pin(response.bytes_stream());
        Ok((stream, response_headers))
    }
}

/// Builder for [`HttpClient`].
pub struct HttpClientBuilder {
    base_url: String,
    headers: HashMap<String, String>,
    connect_timeout: Duration,
    request_timeout: Duration,
}

impl HttpClientBuilder {
    /// Add a default header that will be sent with every request.
    #[must_use]
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set the connection timeout.
    #[must_use]
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set the default request timeout.
    #[must_use]
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Build the [`HttpClient`].
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if headers contain invalid values
    /// or the underlying reqwest client cannot be constructed.
    pub fn build(self) -> SdkResult<HttpClient> {
        let mut header_map = HeaderMap::new();
        for (name, value) in &self.headers {
            let header_name: HeaderName = name.parse().map_err(|e| SdkError::Configuration {
                message: format!("invalid header name '{name}': {e}"),
            })?;
            let header_value: HeaderValue = value.parse().map_err(|e| SdkError::Configuration {
                message: format!("invalid header value for '{name}': {e}"),
            })?;
            header_map.insert(header_name, header_value);
        }

        let client = reqwest::Client::builder()
            .connect_timeout(self.connect_timeout)
            .timeout(self.request_timeout)
            .build()
            .map_err(|e| SdkError::Configuration {
                message: format!("failed to build HTTP client: {e}"),
            })?;

        Ok(HttpClient {
            inner: client,
            base_url: self.base_url,
            default_headers: header_map,
        })
    }
}

/// Extract a human-readable error message from a provider error response.
///
/// Tries common JSON paths (`error.message`, `error.msg`, `message`),
/// then falls back to the raw body text.
fn extract_error_message(json: Option<&serde_json::Value>, raw: &str) -> String {
    if let Some(json) = json {
        // OpenAI / Anthropic: {"error": {"message": "..."}}
        if let Some(msg) = json
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(serde_json::Value::as_str)
        {
            return msg.to_string();
        }
        // Gemini: {"error": {"msg": "..."}} or top-level {"message": "..."}
        if let Some(msg) = json
            .get("error")
            .and_then(|e| e.get("msg"))
            .and_then(serde_json::Value::as_str)
        {
            return msg.to_string();
        }
        if let Some(msg) = json.get("message").and_then(serde_json::Value::as_str) {
            return msg.to_string();
        }
    }

    if raw.is_empty() {
        "unknown error".to_string()
    } else if raw.len() > 200 {
        format!("{}...", &raw[..200])
    } else {
        raw.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_creates_client() -> SdkResult<()> {
        let client = HttpClient::builder("https://api.example.com")
            .header("Authorization", "Bearer test-key")
            .build()?;
        assert_eq!(client.base_url(), "https://api.example.com");
        Ok(())
    }

    #[test]
    fn builder_rejects_invalid_header_name() {
        let result = HttpClient::builder("https://api.example.com")
            .header("invalid header\n", "value")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn extract_openai_error_message() {
        let json: serde_json::Value = serde_json::json!({
            "error": {
                "message": "Invalid API key",
                "type": "authentication_error"
            }
        });
        assert_eq!(extract_error_message(Some(&json), ""), "Invalid API key");
    }

    #[test]
    fn extract_top_level_message() {
        let json: serde_json::Value = serde_json::json!({
            "message": "Something went wrong"
        });
        assert_eq!(
            extract_error_message(Some(&json), ""),
            "Something went wrong"
        );
    }

    #[test]
    fn extract_fallback_to_raw() {
        assert_eq!(
            extract_error_message(None, "raw error text"),
            "raw error text"
        );
    }

    #[test]
    fn extract_empty_body() {
        assert_eq!(extract_error_message(None, ""), "unknown error");
    }

    #[test]
    fn extract_truncates_long_body() {
        let long = "x".repeat(300);
        let result = extract_error_message(None, &long);
        assert!(result.len() < 210);
        assert!(result.ends_with("..."));
    }
}
