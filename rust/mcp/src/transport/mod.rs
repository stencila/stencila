//! Transport layer for MCP communication.
//!
//! The [`Transport`] trait abstracts the bidirectional JSON-RPC 2.0 channel
//! between the client and an MCP server. Implementations handle framing,
//! request/response matching, and server-initiated notification delivery.

pub mod stdio;

use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::error::McpResult;
use crate::types::ServerNotification;

/// Abstraction over the communication channel to an MCP server.
///
/// A transport handles:
/// - Sending JSON-RPC requests and receiving matched responses.
/// - Sending JSON-RPC notifications (fire-and-forget).
/// - Receiving asynchronous server-initiated notifications.
/// - Lifecycle management (connection status, shutdown).
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a JSON-RPC request and wait for the matching response.
    ///
    /// The implementation must correlate the response by `id` and return
    /// [`McpError::Timeout`] if the server does not respond within `timeout`.
    ///
    /// # Errors
    ///
    /// Returns an error if the transport is disconnected, the request cannot
    /// be sent, or the response indicates a JSON-RPC error.
    async fn request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
        timeout: Duration,
    ) -> McpResult<serde_json::Value>;

    /// Send a JSON-RPC notification (no response expected).
    ///
    /// # Errors
    ///
    /// Returns an error if the transport is disconnected or the write fails.
    async fn notify(&self, method: &str, params: Option<serde_json::Value>) -> McpResult<()>;

    /// Take the receiver end of the server-initiated notification channel.
    ///
    /// This can only be called once — subsequent calls return `None`.
    /// The caller uses this to listen for events like `notifications/tools/listChanged`.
    fn take_notification_receiver(&self) -> Option<mpsc::UnboundedReceiver<ServerNotification>>;

    /// Whether the transport is currently connected to the server.
    fn is_connected(&self) -> bool;

    /// Gracefully shut down the transport.
    ///
    /// For stdio transports this closes stdin, waits for the process to exit,
    /// and escalates to SIGTERM/SIGKILL if needed.
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown does not complete cleanly.
    async fn shutdown(&self) -> McpResult<()>;
}
