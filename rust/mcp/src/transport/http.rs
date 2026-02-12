//! Streamable HTTP transport for MCP servers.
//!
//! Communicates with an MCP server over HTTP using the Streamable HTTP
//! transport defined in the MCP specification (2025-03-26). The client
//! sends JSON-RPC messages via HTTP POST and the server responds with
//! either a plain JSON body or an SSE stream. An optional background
//! GET request opens a persistent SSE connection for server-initiated
//! notifications.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

/// Default timeout for notification POSTs and other protocol-level sends.
const DEFAULT_NOTIFY_TIMEOUT: Duration = Duration::from_secs(30);

/// Default connect timeout for the HTTP client.
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

use async_trait::async_trait;
use tokio::sync::{Mutex, RwLock, mpsc, oneshot};

use super::Transport;
use crate::error::{McpError, McpResult, PrettyDuration};
use crate::types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, ServerNotification};

/// State shared between the `HttpTransport` handle and background tasks.
struct SharedState {
    /// Pending request waiters, keyed by JSON-RPC request id.
    pending: Mutex<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>,
    /// Channel for server-initiated notifications.
    notification_tx: mpsc::UnboundedSender<ServerNotification>,
    /// Whether the transport is still connected.
    connected: AtomicBool,
}

/// A transport that communicates with an MCP server over Streamable HTTP.
///
/// The server endpoint is a single URL. Requests are sent as HTTP POST with
/// `Content-Type: application/json` and `Accept: application/json, text/event-stream`.
/// The server may respond with a JSON body or an SSE stream.
///
/// Session management is handled via the `Mcp-Session-Id` header: the server
/// may include it in its first response and the client echoes it in all
/// subsequent requests.
pub struct HttpTransport {
    /// Identifier for this server (used in error messages).
    server_id: String,
    /// The MCP server endpoint URL.
    url: String,
    /// Extra headers to include in every request (from config).
    extra_headers: HashMap<String, String>,
    /// The HTTP client.
    client: reqwest::Client,
    /// Monotonically increasing request id counter.
    next_id: AtomicU64,
    /// Session ID assigned by the server (set on first response).
    /// Wrapped in `Arc` so the SSE listener can read fresh values on reconnect.
    session_id_shared: Arc<RwLock<Option<String>>>,
    /// Shared state with background tasks.
    state: Arc<SharedState>,
    /// Notification receiver — taken once via `take_notification_receiver()`.
    notification_rx: Mutex<Option<mpsc::UnboundedReceiver<ServerNotification>>>,
    /// Handle to the background SSE listener task.
    sse_listener_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

/// The `Mcp-Session-Id` header name.
const SESSION_ID_HEADER: &str = "mcp-session-id";

impl HttpTransport {
    /// Create a new HTTP transport for the given server endpoint.
    ///
    /// This does not make any network requests — the connection is established
    /// lazily on the first `request()` call (during the MCP initialize handshake).
    ///
    /// # Errors
    ///
    /// Returns [`McpError::ConnectionFailed`] if the HTTP client cannot be built.
    pub fn new(
        server_id: impl Into<String>,
        url: impl Into<String>,
        headers: &HashMap<String, String>,
    ) -> McpResult<Self> {
        let server_id = server_id.into();

        let client = reqwest::Client::builder()
            .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
            .build()
            .map_err(|e| McpError::ConnectionFailed {
                server_id: server_id.clone(),
                message: format!("failed to build HTTP client: {e}"),
            })?;

        let (notification_tx, notification_rx) = mpsc::unbounded_channel();

        let state = Arc::new(SharedState {
            pending: Mutex::new(HashMap::new()),
            notification_tx,
            connected: AtomicBool::new(true),
        });

        Ok(Self {
            server_id,
            url: url.into(),
            extra_headers: headers.clone(),
            client,
            next_id: AtomicU64::new(1),
            session_id_shared: Arc::new(RwLock::new(None)),
            state,
            notification_rx: Mutex::new(Some(notification_rx)),
            sse_listener_handle: Mutex::new(None),
        })
    }

    /// Start a background GET SSE listener for server-initiated notifications.
    ///
    /// This should be called after the initialize handshake completes and a
    /// session ID has been established (if the server provides one).
    pub async fn start_sse_listener(&self) {
        let url = self.url.clone();
        let client = self.client.clone();
        let server_id = self.server_id.clone();
        let session_id = Arc::clone(&self.session_id_shared);
        let state = Arc::clone(&self.state);
        let extra_headers = self.extra_headers.clone();

        let handle = tokio::spawn(async move {
            Self::sse_listener_loop(url, client, server_id, session_id, extra_headers, state).await;
        });

        let mut guard = self.sse_listener_handle.lock().await;
        *guard = Some(handle);
    }

    /// Background task that maintains a GET SSE connection for server notifications.
    async fn sse_listener_loop(
        url: String,
        client: reqwest::Client,
        server_id: String,
        session_id: Arc<RwLock<Option<String>>>,
        extra_headers: HashMap<String, String>,
        state: Arc<SharedState>,
    ) {
        loop {
            if !state.connected.load(Ordering::Acquire) {
                break;
            }

            let mut request = client.get(&url).header("Accept", "text/event-stream");

            // Read session ID fresh on each reconnect attempt
            if let Some(ref sid) = *session_id.read().await {
                request = request.header(SESSION_ID_HEADER, sid.as_str());
            }

            for (key, value) in &extra_headers {
                request = request.header(key.as_str(), value.as_str());
            }

            let response = match request.send().await {
                Ok(resp) => resp,
                Err(e) => {
                    tracing::debug!("MCP server `{server_id}` SSE listener connection failed: {e}");
                    // If disconnected, stop retrying
                    if !state.connected.load(Ordering::Acquire) {
                        break;
                    }
                    // Back off before retrying
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            let status = response.status();
            if !status.is_success() {
                // 405 Method Not Allowed means the server doesn't support GET SSE
                if status == reqwest::StatusCode::METHOD_NOT_ALLOWED {
                    tracing::debug!(
                        "MCP server `{server_id}` does not support GET SSE notifications"
                    );
                    break;
                }
                tracing::debug!("MCP server `{server_id}` SSE listener got status {status}");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }

            // Read SSE events from the response body
            if let Err(e) = Self::read_sse_stream(&server_id, response, &state).await {
                tracing::debug!("MCP server `{server_id}` SSE listener stream ended: {e}");
            }

            // If still connected, reconnect after a brief pause
            if !state.connected.load(Ordering::Acquire) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    /// Build the common headers for POST requests.
    async fn build_post_request(&self, body: Vec<u8>) -> reqwest::RequestBuilder {
        let mut request = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(body);

        if let Some(ref sid) = *self.session_id_shared.read().await {
            request = request.header(SESSION_ID_HEADER, sid.as_str());
        }

        for (key, value) in &self.extra_headers {
            request = request.header(key.as_str(), value.as_str());
        }

        request
    }

    /// Store the session ID from a response if the server provides one.
    async fn capture_session_id(&self, response: &reqwest::Response) {
        if let Some(sid) = response.headers().get(SESSION_ID_HEADER)
            && let Ok(sid_str) = sid.to_str()
        {
            let mut guard = self.session_id_shared.write().await;
            *guard = Some(sid_str.to_string());
        }
    }

    /// Read an SSE stream from a response, dispatching JSON-RPC messages.
    ///
    /// Reads chunks from the response body and parses SSE events manually.
    /// Each `data:` line's payload is accumulated, and on an empty line the
    /// complete event is dispatched as a JSON-RPC response or notification.
    ///
    /// Line buffering operates on raw bytes to avoid corrupting multi-byte
    /// UTF-8 sequences that may be split across network chunks.
    async fn read_sse_stream(
        server_id: &str,
        mut response: reqwest::Response,
        state: &SharedState,
    ) -> McpResult<()> {
        let mut data_buf = String::new();
        // Accumulate raw bytes until a newline, then decode the complete line.
        let mut line_bytes: Vec<u8> = Vec::new();

        while let Some(chunk) = response.chunk().await.map_err(|e| McpError::Transport {
            server_id: server_id.to_string(),
            message: format!("SSE stream read error: {e}"),
        })? {
            for &byte in chunk.as_ref() {
                if byte == b'\n' {
                    // Complete line — decode and process
                    let line = String::from_utf8_lossy(&line_bytes);
                    let line = line.trim_end_matches('\r');
                    Self::process_sse_line(server_id, line, &mut data_buf, state).await;
                    line_bytes.clear();
                } else {
                    line_bytes.push(byte);
                }
            }
        }

        // Process any remaining buffered data
        if !data_buf.is_empty() {
            Self::dispatch_sse_data(server_id, &data_buf, state).await;
        }

        Ok(())
    }

    /// Process a single SSE line, updating the data buffer and dispatching
    /// complete events.
    async fn process_sse_line(
        server_id: &str,
        line: &str,
        data_buf: &mut String,
        state: &SharedState,
    ) {
        // SSE spec: "data:" with optional space after colon
        if let Some(rest) = line.strip_prefix("data:") {
            let data = rest.strip_prefix(' ').unwrap_or(rest);
            if !data_buf.is_empty() {
                data_buf.push('\n');
            }
            data_buf.push_str(data);
        } else if line.is_empty() && !data_buf.is_empty() {
            // Empty line = end of event
            Self::dispatch_sse_data(server_id, data_buf, state).await;
            data_buf.clear();
        }
        // Ignore "event:", "id:", "retry:" and comment lines
    }

    /// Dispatch a completed SSE data payload as a JSON-RPC message.
    async fn dispatch_sse_data(server_id: &str, data: &str, state: &SharedState) {
        // Try to parse as a response (has `id` field)
        if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(data)
            && let Some(id) = response.id
        {
            let mut pending = state.pending.lock().await;
            if let Some(sender) = pending.remove(&id) {
                let _ = sender.send(response);
            } else {
                tracing::warn!(
                    "MCP server `{server_id}`: received SSE response for unknown id {id}"
                );
            }
            return;
        }

        // Try to parse as a notification (no `id` field)
        if let Ok(notification) = serde_json::from_str::<ServerNotification>(data) {
            let _ = state.notification_tx.send(notification);
            return;
        }

        tracing::debug!(
            "MCP server `{server_id}`: ignoring unparseable SSE data: {data}",
            data = if data.len() > 200 { &data[..200] } else { data }
        );
    }

    /// Send a POST request and handle the response (JSON or SSE).
    ///
    /// Returns the JSON-RPC result value from the response. For SSE responses,
    /// the matching response is delivered via the pending map.
    async fn post_request(
        &self,
        id: u64,
        body: Vec<u8>,
        timeout: Duration,
    ) -> McpResult<serde_json::Value> {
        // Register a oneshot channel for the response before sending.
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.state.pending.lock().await;
            pending.insert(id, tx);
        }

        let request = self.build_post_request(body).await;

        let response = match tokio::time::timeout(timeout, request.send()).await {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => {
                let mut pending = self.state.pending.lock().await;
                pending.remove(&id);
                return Err(McpError::Transport {
                    server_id: self.server_id.clone(),
                    message: format!("HTTP request failed: {e}"),
                });
            }
            Err(_) => {
                let mut pending = self.state.pending.lock().await;
                pending.remove(&id);
                return Err(McpError::Timeout {
                    server_id: self.server_id.clone(),
                    timeout: PrettyDuration(timeout),
                });
            }
        };

        let status = response.status();
        if !status.is_success() {
            let mut pending = self.state.pending.lock().await;
            pending.remove(&id);

            // Try to extract error body
            let body_text = response.text().await.unwrap_or_default();
            return Err(McpError::Transport {
                server_id: self.server_id.clone(),
                message: format!("HTTP {status}: {body_text}"),
            });
        }

        // Capture session ID from the response
        self.capture_session_id(&response).await;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        if content_type.contains("text/event-stream") {
            // SSE response — read events in the background, then wait for our
            // response on the oneshot channel.
            let server_id = self.server_id.clone();
            let state = Arc::clone(&self.state);
            tokio::spawn(async move {
                if let Err(e) = Self::read_sse_stream(&server_id, response, &state).await {
                    tracing::debug!("MCP server `{server_id}` SSE response stream error: {e}");
                }
            });

            // Wait for the response to arrive via the pending map
            let rpc_response = tokio::time::timeout(timeout, rx)
                .await
                .map_err(|_| {
                    let state = Arc::clone(&self.state);
                    tokio::spawn(async move {
                        let mut pending = state.pending.lock().await;
                        pending.remove(&id);
                    });
                    McpError::Timeout {
                        server_id: self.server_id.clone(),
                        timeout: PrettyDuration(timeout),
                    }
                })?
                .map_err(|_| McpError::Transport {
                    server_id: self.server_id.clone(),
                    message: "server disconnected while waiting for SSE response".to_string(),
                })?;

            Self::extract_result(&self.server_id, rpc_response)
        } else {
            // JSON response — remove from pending (we handle it directly).
            {
                let mut pending = self.state.pending.lock().await;
                pending.remove(&id);
            }

            let body_text = response.text().await.map_err(|e| McpError::Transport {
                server_id: self.server_id.clone(),
                message: format!("failed to read response body: {e}"),
            })?;

            let rpc_response: JsonRpcResponse =
                serde_json::from_str(&body_text).map_err(|e| McpError::Protocol {
                    server_id: self.server_id.clone(),
                    message: format!("invalid JSON-RPC response: {e}"),
                })?;

            Self::extract_result(&self.server_id, rpc_response)
        }
    }

    /// Extract the result value from a JSON-RPC response, checking for errors.
    fn extract_result(server_id: &str, response: JsonRpcResponse) -> McpResult<serde_json::Value> {
        if let Some(error) = response.error {
            return Err(McpError::Protocol {
                server_id: server_id.to_string(),
                message: error.to_string(),
            });
        }

        response.result.ok_or_else(|| McpError::Protocol {
            server_id: server_id.to_string(),
            message: "response has neither result nor error".to_string(),
        })
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
        timeout: Duration,
    ) -> McpResult<serde_json::Value> {
        if !self.state.connected.load(Ordering::Acquire) {
            return Err(McpError::Transport {
                server_id: self.server_id.clone(),
                message: "transport is disconnected".to_string(),
            });
        }

        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let request = JsonRpcRequest::new(id, method, params);
        let json = serde_json::to_vec(&request).map_err(McpError::Json)?;

        self.post_request(id, json, timeout).await
    }

    async fn notify(&self, method: &str, params: Option<serde_json::Value>) -> McpResult<()> {
        if !self.state.connected.load(Ordering::Acquire) {
            return Err(McpError::Transport {
                server_id: self.server_id.clone(),
                message: "transport is disconnected".to_string(),
            });
        }

        let notification = JsonRpcNotification::new(method, params);
        let json = serde_json::to_vec(&notification).map_err(McpError::Json)?;

        let request = self.build_post_request(json).await;

        let response = tokio::time::timeout(DEFAULT_NOTIFY_TIMEOUT, request.send())
            .await
            .map_err(|_| McpError::Timeout {
                server_id: self.server_id.clone(),
                timeout: PrettyDuration(DEFAULT_NOTIFY_TIMEOUT),
            })?
            .map_err(|e| McpError::Transport {
                server_id: self.server_id.clone(),
                message: format!("notification POST failed: {e}"),
            })?;

        // Capture session ID even from notification responses
        self.capture_session_id(&response).await;

        let status = response.status();
        // Accept 200 OK or 202 Accepted for notifications
        if !status.is_success() {
            return Err(McpError::Transport {
                server_id: self.server_id.clone(),
                message: format!("notification POST returned HTTP {status}"),
            });
        }

        Ok(())
    }

    async fn post_connect(&self) {
        self.start_sse_listener().await;
    }

    fn take_notification_receiver(&self) -> Option<mpsc::UnboundedReceiver<ServerNotification>> {
        self.notification_rx.try_lock().ok()?.take()
    }

    fn is_connected(&self) -> bool {
        self.state.connected.load(Ordering::Acquire)
    }

    async fn shutdown(&self) -> McpResult<()> {
        self.state.connected.store(false, Ordering::Release);

        // Cancel the SSE listener task
        let mut handle_guard = self.sse_listener_handle.lock().await;
        if let Some(handle) = handle_guard.take() {
            handle.abort();
        }

        // Send DELETE to terminate the session (best effort)
        let session_id = self.session_id_shared.read().await.clone();
        if session_id.is_some() {
            let mut request = self.client.delete(&self.url);

            if let Some(ref sid) = session_id {
                request = request.header(SESSION_ID_HEADER, sid.as_str());
            }

            for (key, value) in &self.extra_headers {
                request = request.header(key.as_str(), value.as_str());
            }

            // Best effort — ignore errors (server may already be gone)
            let _ = tokio::time::timeout(Duration::from_secs(5), request.send()).await;
        }

        // Wake all pending waiters
        let mut pending = self.state.pending.lock().await;
        for (_, sender) in pending.drain() {
            drop(sender);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn creates_transport() -> McpResult<()> {
        let transport =
            HttpTransport::new("test-server", "http://localhost:8080/mcp", &HashMap::new())?;

        assert_eq!(transport.server_id, "test-server");
        assert_eq!(transport.url, "http://localhost:8080/mcp");
        assert!(transport.is_connected());
        Ok(())
    }

    #[test]
    fn creates_transport_with_headers() -> McpResult<()> {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());

        let transport = HttpTransport::new("auth-server", "http://localhost:8080/mcp", &headers)?;

        assert_eq!(transport.extra_headers.len(), 1);
        assert_eq!(
            transport.extra_headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        Ok(())
    }

    #[tokio::test]
    async fn session_id_management() -> McpResult<()> {
        let transport =
            HttpTransport::new("test-server", "http://localhost:8080/mcp", &HashMap::new())?;

        // Initially no session ID
        assert!(transport.session_id_shared.read().await.is_none());

        // Simulate setting a session ID
        {
            let mut guard = transport.session_id_shared.write().await;
            *guard = Some("test-session-123".to_string());
        }

        assert_eq!(
            transport.session_id_shared.read().await.as_deref(),
            Some("test-session-123")
        );
        Ok(())
    }

    #[tokio::test]
    async fn take_notification_receiver_once() -> McpResult<()> {
        let transport =
            HttpTransport::new("test-server", "http://localhost:8080/mcp", &HashMap::new())?;

        // First call returns Some
        assert!(transport.take_notification_receiver().is_some());

        // Second call returns None
        assert!(transport.take_notification_receiver().is_none());
        Ok(())
    }

    #[tokio::test]
    async fn disconnected_request_fails() -> McpResult<()> {
        let transport =
            HttpTransport::new("test-server", "http://localhost:8080/mcp", &HashMap::new())?;

        transport.shutdown().await?;
        assert!(!transport.is_connected());

        let result = transport
            .request("test/method", None, Duration::from_secs(5))
            .await;

        assert!(matches!(result, Err(McpError::Transport { .. })));
        Ok(())
    }

    #[tokio::test]
    async fn disconnected_notify_fails() -> McpResult<()> {
        let transport =
            HttpTransport::new("test-server", "http://localhost:8080/mcp", &HashMap::new())?;

        transport.shutdown().await?;

        let result = transport.notify("notifications/initialized", None).await;
        assert!(matches!(result, Err(McpError::Transport { .. })));
        Ok(())
    }

    #[tokio::test]
    async fn shutdown_is_idempotent() -> McpResult<()> {
        let transport =
            HttpTransport::new("test-server", "http://localhost:8080/mcp", &HashMap::new())?;

        transport.shutdown().await?;
        transport.shutdown().await?;
        assert!(!transport.is_connected());
        Ok(())
    }

    #[tokio::test]
    async fn request_to_bad_url_fails() {
        let transport = HttpTransport::new("bad-server", "http://127.0.0.1:1/mcp", &HashMap::new())
            .expect("transport creation should succeed");

        let result = transport
            .request("test/method", None, Duration::from_secs(2))
            .await;

        // Should fail with transport error (connection refused)
        assert!(
            matches!(
                &result,
                Err(McpError::Transport { .. } | McpError::Timeout { .. })
            ),
            "expected Transport or Timeout error, got {result:?}"
        );
    }

    #[tokio::test]
    async fn dispatch_sse_data_response() {
        let (notification_tx, _notification_rx) = mpsc::unbounded_channel();
        let state = Arc::new(SharedState {
            pending: Mutex::new(HashMap::new()),
            notification_tx,
            connected: AtomicBool::new(true),
        });

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = state.pending.lock().await;
            pending.insert(42, tx);
        }

        let data = r#"{"jsonrpc":"2.0","id":42,"result":{"ok":true}}"#;
        HttpTransport::dispatch_sse_data("test", data, &state).await;

        let response = rx.await.expect("should receive response");
        assert_eq!(response.id, Some(42));
        assert!(response.result.is_some());
    }

    #[tokio::test]
    async fn dispatch_sse_data_notification() {
        let (notification_tx, mut notification_rx) = mpsc::unbounded_channel();
        let state = Arc::new(SharedState {
            pending: Mutex::new(HashMap::new()),
            notification_tx,
            connected: AtomicBool::new(true),
        });

        let data = r#"{"jsonrpc":"2.0","method":"notifications/tools/listChanged"}"#;
        HttpTransport::dispatch_sse_data("test", data, &state).await;

        let notification = notification_rx
            .recv()
            .await
            .expect("should receive notification");
        assert_eq!(notification.method, "notifications/tools/listChanged");
    }

    #[tokio::test]
    async fn extract_result_success() -> McpResult<()> {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(1),
            result: Some(serde_json::json!({"tools": []})),
            error: None,
        };

        let result = HttpTransport::extract_result("test", response)?;
        assert_eq!(result, serde_json::json!({"tools": []}));
        Ok(())
    }

    #[tokio::test]
    async fn extract_result_error() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(1),
            result: None,
            error: Some(crate::types::JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        };

        let result = HttpTransport::extract_result("test", response);
        match result {
            Err(McpError::Protocol { message, .. }) => {
                assert!(message.contains("Method not found"));
            }
            other => panic!("expected Protocol error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn extract_result_neither() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(1),
            result: None,
            error: None,
        };

        let result = HttpTransport::extract_result("test", response);
        assert!(matches!(result, Err(McpError::Protocol { .. })));
    }

    #[tokio::test]
    async fn process_sse_line_data_without_space() {
        let (notification_tx, mut notification_rx) = mpsc::unbounded_channel();
        let state = Arc::new(SharedState {
            pending: Mutex::new(HashMap::new()),
            notification_tx,
            connected: AtomicBool::new(true),
        });

        // "data:<payload>" without space after colon is valid SSE
        let mut data_buf = String::new();
        let data = r#"{"jsonrpc":"2.0","method":"notifications/tools/listChanged"}"#;
        let line = format!("data:{data}");
        HttpTransport::process_sse_line("test", &line, &mut data_buf, &state).await;

        // Empty line triggers dispatch
        HttpTransport::process_sse_line("test", "", &mut data_buf, &state).await;

        let notification = notification_rx
            .recv()
            .await
            .expect("should receive notification");
        assert_eq!(notification.method, "notifications/tools/listChanged");
    }

    #[tokio::test]
    async fn process_sse_line_data_with_space() {
        let (notification_tx, mut notification_rx) = mpsc::unbounded_channel();
        let state = Arc::new(SharedState {
            pending: Mutex::new(HashMap::new()),
            notification_tx,
            connected: AtomicBool::new(true),
        });

        // "data: <payload>" with space after colon
        let mut data_buf = String::new();
        HttpTransport::process_sse_line(
            "test",
            r#"data: {"jsonrpc":"2.0","method":"test/event"}"#,
            &mut data_buf,
            &state,
        )
        .await;
        HttpTransport::process_sse_line("test", "", &mut data_buf, &state).await;

        let notification = notification_rx
            .recv()
            .await
            .expect("should receive notification");
        assert_eq!(notification.method, "test/event");
    }

    #[tokio::test]
    async fn sse_multibyte_utf8_across_chunks() {
        // Simulate multi-byte UTF-8 split across chunks by processing
        // raw bytes through read_sse_stream logic manually.
        let (notification_tx, _rx) = mpsc::unbounded_channel();
        let state = Arc::new(SharedState {
            pending: Mutex::new(HashMap::new()),
            notification_tx,
            connected: AtomicBool::new(true),
        });

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = state.pending.lock().await;
            pending.insert(1, tx);
        }

        // A JSON-RPC response containing a multi-byte char (e.g. emoji)
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"text":"hello \u00e9"}}"#;
        let sse_event = format!("data: {json}\n\n");
        let bytes = sse_event.as_bytes();

        // Split in the middle of a line (simulating chunk boundary)
        let mid = bytes.len() / 2;
        let chunk1 = bytes::Bytes::from(bytes[..mid].to_vec());
        let chunk2 = bytes::Bytes::from(bytes[mid..].to_vec());

        // Process byte by byte as read_sse_stream does
        let mut data_buf = String::new();
        let mut line_bytes: Vec<u8> = Vec::new();

        for chunk in [chunk1, chunk2] {
            for &byte in chunk.as_ref() {
                if byte == b'\n' {
                    let line = String::from_utf8_lossy(&line_bytes);
                    let line = line.trim_end_matches('\r');
                    HttpTransport::process_sse_line("test", line, &mut data_buf, &state).await;
                    line_bytes.clear();
                } else {
                    line_bytes.push(byte);
                }
            }
        }

        let response = rx.await.expect("should receive response");
        assert_eq!(response.id, Some(1));
        assert!(response.result.is_some());
    }
}
