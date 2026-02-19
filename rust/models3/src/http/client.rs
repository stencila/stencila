//! HTTP client wrapper for provider API calls.
//!
//! Wraps [`reqwest::Client`] with LLM-provider-friendly defaults:
//! timeouts, base URL, default headers, and convenience methods for
//! JSON POST and streaming POST (the two request patterns used by
//! every provider).

use std::collections::HashMap;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use bytes::Bytes;
use futures::stream::Stream;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::error::{SdkError, SdkResult};
use crate::types::timeout::Timeout;

/// Error type for the byte stream returned by [`HttpClient::post_stream`].
///
/// Wraps transport errors from reqwest and adds an idle-timeout variant so
/// that stream consumers can distinguish a timeout from a normal EOF.
#[derive(Debug, thiserror::Error)]
pub enum ByteStreamError {
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("stream idle timeout after {idle_secs:.1}s")]
    IdleTimeout { idle_secs: f64 },
}

/// A boxed byte stream with [`ByteStreamError`] as the error type.
///
/// This is the stream type returned by [`HttpClient::post_stream`] and
/// accepted by [`super::sse::parse_sse`].
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, ByteStreamError>> + Send>>;

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
    /// `Timeout.request` is applied as a per-request deadline.
    /// `Timeout.stream_idle` is enforced by wrapping the byte stream with
    /// an [`IdleTimeoutStream`] that yields a [`ByteStreamError::IdleTimeout`]
    /// error if no chunk arrives within the configured duration.
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
    ) -> SdkResult<(ByteStream, HeaderMap)> {
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

        use futures::StreamExt;
        let stream: ByteStream = Box::pin(
            response
                .bytes_stream()
                .map(|r| r.map_err(ByteStreamError::from)),
        );

        let stream: ByteStream = if let Some(t) = timeout
            && let Some(secs) = t.stream_idle
            && secs.is_finite()
            && secs > 0.0
        {
            Box::pin(IdleTimeoutStream::new(
                stream,
                Duration::from_secs_f64(secs),
            ))
        } else {
            stream
        };

        Ok((stream, response_headers))
    }
}

/// A stream wrapper that enforces an idle timeout between chunks.
///
/// If no item arrives from the inner stream within `idle_duration`, the
/// stream yields a [`ByteStreamError::IdleTimeout`] error, then terminates.
/// This surfaces the timeout as an explicit error rather than silent EOF,
/// allowing downstream consumers (e.g. the SSE parser) to report it.
///
/// The timeout is reset each time the inner stream yields an item.
struct IdleTimeoutStream {
    inner: ByteStream,
    idle_duration: Duration,
    deadline: Pin<Box<tokio::time::Sleep>>,
    timed_out: bool,
}

impl IdleTimeoutStream {
    fn new(inner: ByteStream, idle_duration: Duration) -> Self {
        Self {
            inner,
            idle_duration,
            deadline: Box::pin(tokio::time::sleep(idle_duration)),
            timed_out: false,
        }
    }
}

impl Stream for IdleTimeoutStream {
    type Item = Result<Bytes, ByteStreamError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.timed_out {
            return Poll::Ready(None);
        }

        // Poll the inner stream first.
        match this.inner.as_mut().poll_next(cx) {
            Poll::Ready(item) => {
                // Reset the deadline for the next chunk.
                this.deadline
                    .as_mut()
                    .reset(tokio::time::Instant::now() + this.idle_duration);
                Poll::Ready(item)
            }
            Poll::Pending => {
                // Inner stream is not ready — check the idle deadline.
                match this.deadline.as_mut().poll(cx) {
                    Poll::Ready(()) => {
                        let idle_secs = this.idle_duration.as_secs_f64();
                        tracing::warn!(
                            idle_secs,
                            "stream idle timeout expired, terminating byte stream"
                        );
                        this.timed_out = true;
                        Poll::Ready(Some(Err(ByteStreamError::IdleTimeout { idle_secs })))
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
        }
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
        // ChatGPT backend: {"detail": "..."}
        if let Some(msg) = json.get("detail").and_then(serde_json::Value::as_str) {
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
    use futures::StreamExt;
    use std::task::Poll;

    fn mock_bytes_stream(chunks: Vec<&'static str>) -> ByteStream {
        Box::pin(futures::stream::iter(
            chunks
                .into_iter()
                .map(|c| Ok(Bytes::from(c)))
                .collect::<Vec<_>>(),
        ))
    }

    #[tokio::test]
    async fn idle_timeout_passes_through_fast_stream() {
        let inner = mock_bytes_stream(vec!["hello", " ", "world"]);
        let stream = IdleTimeoutStream::new(inner, Duration::from_secs(10));
        tokio::pin!(stream);
        let mut collected = Vec::new();
        while let Some(item) = stream.next().await {
            collected.push(item.unwrap());
        }
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], "hello");
        assert_eq!(collected[2], "world");
    }

    #[tokio::test]
    async fn idle_timeout_yields_error_then_ends() {
        // A stream that yields one item then hangs forever
        let inner: ByteStream = Box::pin(
            futures::stream::iter(vec![Ok(Bytes::from("first"))]).chain(futures::stream::poll_fn(
                |_cx| -> Poll<Option<Result<Bytes, ByteStreamError>>> { Poll::Pending },
            )),
        );

        let stream = IdleTimeoutStream::new(inner, Duration::from_millis(50));
        tokio::pin!(stream);

        // First item should arrive
        let first = stream.next().await;
        assert!(first.is_some());
        assert_eq!(first.unwrap().unwrap(), "first");

        // Second poll should time out — yielding an IdleTimeout error
        let second = stream.next().await;
        assert!(second.is_some(), "timeout should yield an error item");
        let err = second.unwrap().unwrap_err();
        assert!(
            matches!(err, ByteStreamError::IdleTimeout { .. }),
            "expected IdleTimeout, got: {err:?}"
        );

        // After the error, stream should be done (returns None)
        let third = stream.next().await;
        assert!(third.is_none());
    }

    #[tokio::test]
    async fn idle_timeout_stays_none_after_timeout() {
        let inner: ByteStream = Box::pin(
            futures::stream::iter(vec![Ok(Bytes::from("data"))]).chain(futures::stream::poll_fn(
                |_cx| -> Poll<Option<Result<Bytes, ByteStreamError>>> { Poll::Pending },
            )),
        );

        let stream = IdleTimeoutStream::new(inner, Duration::from_millis(20));
        tokio::pin!(stream);

        let _ = stream.next().await; // consume "data"
        let _ = stream.next().await; // times out -> Err(IdleTimeout)

        // Subsequent polls should return None
        let third = stream.next().await;
        assert!(third.is_none());
    }

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
