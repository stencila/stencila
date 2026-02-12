//! Stdio transport for MCP servers.
//!
//! Spawns a child process and communicates via JSON-RPC 2.0 over stdin/stdout.
//! A background reader task dispatches responses by `id` and forwards
//! server-initiated notifications to a channel.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, mpsc, oneshot};

use super::Transport;
use crate::error::{McpError, McpResult, PrettyDuration};
use crate::types::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, ServerNotification};

/// How long to wait after closing stdin before force-killing the process.
const SHUTDOWN_GRACEFUL_MS: u64 = 4000;

/// State shared between the `StdioTransport` handle and the background reader.
struct SharedState {
    /// Pending request waiters, keyed by JSON-RPC request id.
    pending: Mutex<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>,
    /// Channel for server-initiated notifications.
    notification_tx: mpsc::UnboundedSender<ServerNotification>,
    /// Whether the transport is still connected.
    connected: AtomicBool,
}

/// A transport that communicates with an MCP server over stdin/stdout.
///
/// The server process is spawned on creation. A background tokio task reads
/// stdout line-by-line, dispatching JSON-RPC responses to waiting callers and
/// forwarding notifications to a channel.
pub struct StdioTransport {
    /// Identifier for this server (used in error messages).
    server_id: String,
    /// Monotonically increasing request id counter.
    next_id: AtomicU64,
    /// Handle to the child process's stdin (write half).
    stdin: Mutex<Option<tokio::process::ChildStdin>>,
    /// Handle to the child process (for shutdown).
    child: Mutex<Option<Child>>,
    /// Shared state with the background reader.
    state: Arc<SharedState>,
    /// Notification receiver — taken once via `take_notification_receiver()`.
    notification_rx: Mutex<Option<mpsc::UnboundedReceiver<ServerNotification>>>,
    /// Handle to the background reader task.
    reader_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl StdioTransport {
    /// Spawn a child process and create a new stdio transport.
    ///
    /// The `command` is the executable to run, `args` are command-line arguments,
    /// and `env` provides additional environment variables for the child process.
    ///
    /// # Errors
    ///
    /// Returns [`McpError::ConnectionFailed`] if the process cannot be spawned.
    pub fn spawn(
        server_id: impl Into<String>,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> McpResult<Self> {
        let server_id = server_id.into();

        let mut cmd = Command::new(command);
        cmd.args(args)
            .envs(env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn().map_err(|e| McpError::ConnectionFailed {
            server_id: server_id.clone(),
            message: format!("failed to spawn `{command}`: {e}"),
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::ConnectionFailed {
                server_id: server_id.clone(),
                message: "child process stdin not available".to_string(),
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::ConnectionFailed {
                server_id: server_id.clone(),
                message: "child process stdout not available".to_string(),
            })?;

        // Drain stderr in a background task to prevent pipe buffer deadlocks.
        // MCP servers commonly log to stderr; if the buffer fills the server blocks.
        if let Some(stderr) = child.stderr.take() {
            let stderr_server_id = server_id.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::debug!("MCP server `{stderr_server_id}` stderr: {line}");
                }
            });
        }

        let (notification_tx, notification_rx) = mpsc::unbounded_channel();

        let state = Arc::new(SharedState {
            pending: Mutex::new(HashMap::new()),
            notification_tx,
            connected: AtomicBool::new(true),
        });

        // Spawn background reader task.
        let reader_state = Arc::clone(&state);
        let reader_server_id = server_id.clone();
        let reader_handle = tokio::spawn(async move {
            Self::reader_loop(reader_server_id, stdout, reader_state).await;
        });

        Ok(Self {
            server_id,
            next_id: AtomicU64::new(1),
            stdin: Mutex::new(Some(stdin)),
            child: Mutex::new(Some(child)),
            state,
            notification_rx: Mutex::new(Some(notification_rx)),
            reader_handle: Mutex::new(Some(reader_handle)),
        })
    }

    /// Background task that reads stdout line-by-line, dispatching responses
    /// and notifications.
    async fn reader_loop(
        server_id: String,
        stdout: tokio::process::ChildStdout,
        state: Arc<SharedState>,
    ) {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    let line = line.trim().to_string();
                    if line.is_empty() {
                        continue;
                    }
                    Self::dispatch_line(&server_id, &line, &state).await;
                }
                Ok(None) => {
                    // EOF — process exited.
                    tracing::debug!("MCP server `{server_id}` stdout closed (EOF)");
                    break;
                }
                Err(e) => {
                    tracing::warn!("MCP server `{server_id}` stdout read error: {e}");
                    break;
                }
            }
        }

        // Mark disconnected and wake all pending waiters with an error-like signal.
        state.connected.store(false, Ordering::Release);
        let mut pending = state.pending.lock().await;
        for (_, sender) in pending.drain() {
            // Drop the sender — the receiver will get a `RecvError`.
            drop(sender);
        }
    }

    /// Dispatch a single line from stdout as either a response or a notification.
    async fn dispatch_line(server_id: &str, line: &str, state: &SharedState) {
        // Try to parse as a response (has `id` field).
        if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(line)
            && let Some(id) = response.id
        {
            let mut pending = state.pending.lock().await;
            if let Some(sender) = pending.remove(&id) {
                // Ignore send failure — caller may have timed out.
                let _ = sender.send(response);
            } else {
                tracing::warn!("MCP server `{server_id}`: received response for unknown id {id}");
            }
            return;
        }

        // Try to parse as a notification (no `id` field).
        if let Ok(notification) = serde_json::from_str::<ServerNotification>(line) {
            let _ = state.notification_tx.send(notification);
            return;
        }

        tracing::debug!(
            "MCP server `{server_id}`: ignoring unparseable line: {line}",
            line = if line.len() > 200 { &line[..200] } else { line }
        );
    }

    /// Write a JSON line to the child process's stdin.
    async fn write_line(&self, json: &[u8]) -> McpResult<()> {
        let mut stdin_guard = self.stdin.lock().await;
        let stdin = stdin_guard.as_mut().ok_or_else(|| McpError::Transport {
            server_id: self.server_id.clone(),
            message: "stdin closed".to_string(),
        })?;

        stdin
            .write_all(json)
            .await
            .map_err(|e| McpError::Transport {
                server_id: self.server_id.clone(),
                message: format!("stdin write failed: {e}"),
            })?;

        stdin
            .write_all(b"\n")
            .await
            .map_err(|e| McpError::Transport {
                server_id: self.server_id.clone(),
                message: format!("stdin write newline failed: {e}"),
            })?;

        stdin.flush().await.map_err(|e| McpError::Transport {
            server_id: self.server_id.clone(),
            message: format!("stdin flush failed: {e}"),
        })?;

        Ok(())
    }

    /// Shut down the child process with escalation:
    /// 1. Close stdin and wait for the process to exit gracefully.
    /// 2. Force kill if it doesn't exit in time.
    async fn shutdown_child(
        server_id: &str,
        child: &mut Child,
        stdin: &mut Option<tokio::process::ChildStdin>,
        graceful_ms: u64,
    ) -> McpResult<()> {
        // Step 1: close stdin — many well-behaved servers exit on EOF.
        *stdin = None;

        if let Ok(Ok(_status)) =
            tokio::time::timeout(Duration::from_millis(graceful_ms), child.wait()).await
        {
            return Ok(());
        }

        // Step 2: Force kill.
        child.kill().await.map_err(|e| McpError::Transport {
            server_id: server_id.to_string(),
            message: format!("failed to kill process: {e}"),
        })?;

        // Reap the zombie.
        let _ = child.wait().await;

        Ok(())
    }
}

#[async_trait]
impl Transport for StdioTransport {
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

        // Register a oneshot channel for the response before writing.
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.state.pending.lock().await;
            pending.insert(id, tx);
        }

        // Write the request.
        if let Err(e) = self.write_line(&json).await {
            // Clean up pending entry on write failure.
            let mut pending = self.state.pending.lock().await;
            pending.remove(&id);
            return Err(e);
        }

        // Wait for the response with a timeout.
        let response = tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| {
                // Timeout — clean up pending entry.
                let state = Arc::clone(&self.state);
                let id_to_remove = id;
                tokio::spawn(async move {
                    let mut pending = state.pending.lock().await;
                    pending.remove(&id_to_remove);
                });
                McpError::Timeout {
                    server_id: self.server_id.clone(),
                    timeout: PrettyDuration(timeout),
                }
            })?
            .map_err(|_| McpError::Transport {
                server_id: self.server_id.clone(),
                message: "server disconnected while waiting for response".to_string(),
            })?;

        // Check for JSON-RPC error.
        if let Some(error) = response.error {
            return Err(McpError::Protocol {
                server_id: self.server_id.clone(),
                message: error.to_string(),
            });
        }

        response.result.ok_or_else(|| McpError::Protocol {
            server_id: self.server_id.clone(),
            message: "response has neither result nor error".to_string(),
        })
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
        self.write_line(&json).await
    }

    fn take_notification_receiver(&self) -> Option<mpsc::UnboundedReceiver<ServerNotification>> {
        // We use try_lock here to avoid blocking — this is expected to succeed
        // since take is called once during setup.
        self.notification_rx.try_lock().ok()?.take()
    }

    fn is_connected(&self) -> bool {
        self.state.connected.load(Ordering::Acquire)
    }

    async fn shutdown(&self) -> McpResult<()> {
        // Mark disconnected first so no new requests are accepted.
        self.state.connected.store(false, Ordering::Release);

        // Shut down the child process.
        let mut child_guard = self.child.lock().await;
        let mut stdin_guard = self.stdin.lock().await;

        if let Some(ref mut child) = *child_guard {
            Self::shutdown_child(
                &self.server_id,
                child,
                &mut stdin_guard,
                SHUTDOWN_GRACEFUL_MS,
            )
            .await?;
        }

        // Drop stdin explicitly.
        *stdin_guard = None;
        *child_guard = None;

        // Cancel the reader task.
        let mut handle_guard = self.reader_handle.lock().await;
        if let Some(handle) = handle_guard.take() {
            handle.abort();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: spawn a simple echo server that reads JSON-RPC requests from stdin
    /// and writes back responses on stdout.
    fn spawn_echo_transport() -> McpResult<StdioTransport> {
        // A small Python script that echoes JSON-RPC responses.
        let script = r#"
import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        msg = json.loads(line)
    except json.JSONDecodeError:
        continue
    # If it has an id, it's a request — echo back the params as the result.
    if "id" in msg:
        resp = {"jsonrpc": "2.0", "id": msg["id"], "result": msg.get("params", {})}
        sys.stdout.write(json.dumps(resp) + "\n")
        sys.stdout.flush()
    # If it's a notification, ignore it.
"#;

        StdioTransport::spawn(
            "echo-server",
            "python3",
            &["-c".to_string(), script.to_string()],
            &HashMap::new(),
        )
    }

    #[tokio::test]
    async fn request_response_round_trip() -> McpResult<()> {
        let transport = spawn_echo_transport()?;

        let params = serde_json::json!({"key": "value"});
        let result = transport
            .request("test/method", Some(params.clone()), Duration::from_secs(5))
            .await?;

        assert_eq!(result, params);

        transport.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn multiple_concurrent_requests() -> McpResult<()> {
        let transport = Arc::new(spawn_echo_transport()?);

        let mut handles = Vec::new();
        for i in 0..10 {
            let t = Arc::clone(&transport);
            handles.push(tokio::spawn(async move {
                let params = serde_json::json!({"index": i});
                let result = t
                    .request(
                        "test/concurrent",
                        Some(params.clone()),
                        Duration::from_secs(5),
                    )
                    .await?;
                assert_eq!(result["index"], i);
                Ok::<_, McpError>(())
            }));
        }

        for handle in handles {
            handle.await.map_err(|e| McpError::Transport {
                server_id: "test".into(),
                message: format!("join error: {e}"),
            })??;
        }

        transport.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn request_timeout() -> McpResult<()> {
        // A server that never responds.
        let script = r"
import sys, time
# Read stdin but never write back.
for line in sys.stdin:
    pass
";
        let transport = StdioTransport::spawn(
            "slow-server",
            "python3",
            &["-c".to_string(), script.to_string()],
            &HashMap::new(),
        )?;

        let result = transport
            .request("test/slow", None, Duration::from_millis(100))
            .await;

        assert!(matches!(result, Err(McpError::Timeout { .. })));

        transport.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn notification_forwarding() -> McpResult<()> {
        // A server that sends a notification after receiving a request.
        let script = r#"
import sys, json

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    msg = json.loads(line)
    if "id" in msg:
        # First send a notification.
        notif = {"jsonrpc": "2.0", "method": "notifications/tools/listChanged"}
        sys.stdout.write(json.dumps(notif) + "\n")
        sys.stdout.flush()
        # Then the response.
        resp = {"jsonrpc": "2.0", "id": msg["id"], "result": {"ok": True}}
        sys.stdout.write(json.dumps(resp) + "\n")
        sys.stdout.flush()
        break
"#;
        let transport = StdioTransport::spawn(
            "notifier",
            "python3",
            &["-c".to_string(), script.to_string()],
            &HashMap::new(),
        )?;

        let mut rx = transport
            .take_notification_receiver()
            .ok_or_else(|| McpError::Transport {
                server_id: "test".into(),
                message: "no notification receiver".into(),
            })?;

        // Second call returns None.
        assert!(transport.take_notification_receiver().is_none());

        // Send a request to trigger the notification.
        let result = transport
            .request("trigger", None, Duration::from_secs(5))
            .await?;
        assert_eq!(result["ok"], true);

        // The notification should arrive on the channel.
        let notification = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .map_err(|_| McpError::Transport {
                server_id: "test".into(),
                message: "timeout waiting for notification".into(),
            })?
            .ok_or_else(|| McpError::Transport {
                server_id: "test".into(),
                message: "notification channel closed".into(),
            })?;

        assert_eq!(notification.method, "notifications/tools/listChanged");

        transport.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn disconnect_on_process_exit() -> McpResult<()> {
        // A server that exits immediately.
        let script = r"
import sys
sys.exit(0)
";
        let transport = StdioTransport::spawn(
            "exiter",
            "python3",
            &["-c".to_string(), script.to_string()],
            &HashMap::new(),
        )?;

        // Give the process a moment to exit.
        tokio::time::sleep(Duration::from_millis(200)).await;

        assert!(!transport.is_connected());

        // Requests should fail with a transport error.
        let result = transport
            .request("test", None, Duration::from_secs(1))
            .await;
        assert!(matches!(result, Err(McpError::Transport { .. })));

        transport.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn shutdown_is_idempotent() -> McpResult<()> {
        let transport = spawn_echo_transport()?;
        transport.shutdown().await?;
        // Second shutdown should not panic or error.
        transport.shutdown().await?;
        assert!(!transport.is_connected());
        Ok(())
    }

    #[tokio::test]
    async fn notify_fire_and_forget() -> McpResult<()> {
        let transport = spawn_echo_transport()?;

        // Notifications should not error even though the server ignores them.
        transport
            .notify("notifications/initialized", Some(serde_json::json!({})))
            .await?;

        transport.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn json_rpc_error_response() -> McpResult<()> {
        // A server that returns a JSON-RPC error for every request.
        let script = r#"
import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    msg = json.loads(line)
    if "id" in msg:
        resp = {
            "jsonrpc": "2.0",
            "id": msg["id"],
            "error": {"code": -32601, "message": "Method not found"}
        }
        sys.stdout.write(json.dumps(resp) + "\n")
        sys.stdout.flush()
"#;
        let transport = StdioTransport::spawn(
            "error-server",
            "python3",
            &["-c".to_string(), script.to_string()],
            &HashMap::new(),
        )?;

        let result = transport
            .request("nonexistent", None, Duration::from_secs(5))
            .await;

        match result {
            Err(McpError::Protocol { message, .. }) => {
                assert!(message.contains("Method not found"));
            }
            other => panic!("expected Protocol error, got {other:?}"),
        }

        transport.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn spawn_nonexistent_command_fails() {
        let result = StdioTransport::spawn(
            "bad-server",
            "this-command-definitely-does-not-exist-12345",
            &[],
            &HashMap::new(),
        );
        assert!(matches!(result, Err(McpError::ConnectionFailed { .. })));
    }

    #[tokio::test]
    async fn env_vars_passed_to_child() -> McpResult<()> {
        // A server that reads an env var and returns it.
        let script = r#"
import sys, json, os
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    msg = json.loads(line)
    if "id" in msg:
        val = os.environ.get("MCP_TEST_VAR", "NOT_SET")
        resp = {"jsonrpc": "2.0", "id": msg["id"], "result": {"env_val": val}}
        sys.stdout.write(json.dumps(resp) + "\n")
        sys.stdout.flush()
        break
"#;
        let mut env = HashMap::new();
        env.insert("MCP_TEST_VAR".to_string(), "hello_mcp".to_string());

        let transport = StdioTransport::spawn(
            "env-server",
            "python3",
            &["-c".to_string(), script.to_string()],
            &env,
        )?;

        let result = transport
            .request("get_env", None, Duration::from_secs(5))
            .await?;

        assert_eq!(result["env_val"], "hello_mcp");

        transport.shutdown().await?;
        Ok(())
    }
}
