//! Live MCP server connection.
//!
//! [`LiveMcpServer`] wraps a [`Transport`] with MCP protocol handling:
//! initialize handshake, tool listing with caching, tool execution, and
//! server-initiated notification listening.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    McpError, McpResult,
    config::{McpServerConfig, TransportConfig},
    traits::{McpContent, McpServer, McpToolInfo, McpToolResult},
    transport::Transport,
    transport::http::HttpTransport,
    transport::stdio::StdioTransport,
    types::{McpContentBlock, ServerCapabilities, ServerInfo, ToolCallResult, ToolsListResult},
};

/// Default timeout for tool calls.
const DEFAULT_TOOL_TIMEOUT: Duration = Duration::from_secs(30);

/// Default timeout for protocol operations (initialize, tools/list).
const DEFAULT_PROTOCOL_TIMEOUT: Duration = Duration::from_secs(15);

/// MCP protocol version.
const PROTOCOL_VERSION: &str = "2025-03-26";

/// A live connection to an MCP server.
///
/// Performs the MCP initialize handshake on connect, caches tool definitions,
/// and listens for `notifications/tools/listChanged` to invalidate the cache.
// Cannot derive Debug because `Box<dyn Transport>` is not Debug.
pub struct LiveMcpServer {
    /// Server configuration.
    config: McpServerConfig,

    /// Transport layer for JSON-RPC communication.
    transport: Box<dyn Transport>,

    /// Server identity from the initialize response.
    server_info: Option<ServerInfo>,

    /// Server capabilities from the initialize response.
    server_capabilities: ServerCapabilities,

    /// Cached tool definitions (populated on first `tools()` call).
    tools_cache: RwLock<Option<Vec<McpToolInfo>>>,

    /// Set to `true` by the notification listener when tools change.
    tools_dirty: Arc<AtomicBool>,

    /// Timeout for `tools/call` requests.
    tool_timeout: Duration,
}

impl std::fmt::Debug for LiveMcpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveMcpServer")
            .field("server_id", &self.config.id)
            .field("connected", &self.transport.is_connected())
            .finish_non_exhaustive()
    }
}

impl LiveMcpServer {
    /// Connect to an MCP server using the provided configuration.
    ///
    /// Spawns the server process (for stdio transport), performs the MCP
    /// initialize handshake, sends `notifications/initialized`, and starts
    /// listening for server-initiated notifications.
    ///
    /// # Errors
    ///
    /// Returns [`McpError::ConnectionFailed`] if the process cannot be spawned
    /// or the initialize handshake fails.
    pub async fn connect(config: McpServerConfig) -> McpResult<Self> {
        let transport: Box<dyn Transport> = match &config.transport {
            TransportConfig::Stdio { command, args } => Box::new(StdioTransport::spawn(
                &config.id,
                command,
                args,
                &config.env,
            )?),
            TransportConfig::Http { url, headers } => {
                Box::new(HttpTransport::new(&config.id, url, headers)?)
            }
        };

        Self::connect_with_transport(config, transport).await
    }

    /// Connect using a pre-built transport.
    ///
    /// Useful for testing with mock transports.
    pub(crate) async fn connect_with_transport(
        config: McpServerConfig,
        transport: Box<dyn Transport>,
    ) -> McpResult<Self> {
        // MCP initialize handshake
        let init_params = serde_json::json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {
                "name": "stencila",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        let result = transport
            .request("initialize", Some(init_params), DEFAULT_PROTOCOL_TIMEOUT)
            .await
            .map_err(|e| McpError::ConnectionFailed {
                server_id: config.id.clone(),
                message: format!("initialize handshake failed: {e}"),
            })?;

        let init_result: crate::types::InitializeResult =
            serde_json::from_value(result).map_err(|e| McpError::Protocol {
                server_id: config.id.clone(),
                message: format!("invalid initialize response: {e}"),
            })?;

        // Send initialized notification
        transport.notify("notifications/initialized", None).await?;

        // Allow transport to start background tasks now that the handshake
        // is complete and session ID is established.
        transport.post_connect().await;

        let server_info = init_result.server_info;
        let server_capabilities = init_result.capabilities;
        let tools_dirty = Arc::new(AtomicBool::new(false));

        let mut server = Self {
            config,
            transport,
            server_info,
            server_capabilities,
            tools_cache: RwLock::new(None),
            tools_dirty,
            tool_timeout: DEFAULT_TOOL_TIMEOUT,
        };

        server.start_notification_listener();

        Ok(server)
    }

    /// Set the timeout for `tools/call` requests.
    pub fn set_tool_timeout(&mut self, timeout: Duration) {
        self.tool_timeout = timeout;
    }

    /// Whether the underlying transport is still connected.
    pub fn is_connected(&self) -> bool {
        self.transport.is_connected()
    }

    /// Shut down the server connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the transport shutdown does not complete cleanly.
    pub async fn shutdown(&self) -> McpResult<()> {
        self.transport.shutdown().await
    }

    /// Fetch tool definitions from the server via `tools/list`.
    async fn fetch_tools(&self) -> McpResult<Vec<McpToolInfo>> {
        let result = self
            .transport
            .request("tools/list", None, DEFAULT_PROTOCOL_TIMEOUT)
            .await?;

        let tools_result: ToolsListResult =
            serde_json::from_value(result).map_err(|e| McpError::Protocol {
                server_id: self.config.id.clone(),
                message: format!("invalid tools/list response: {e}"),
            })?;

        Ok(tools_result
            .tools
            .into_iter()
            .map(|t| McpToolInfo {
                name: t.name,
                description: t.description,
                input_schema: t.input_schema,
                output_schema: t.output_schema,
                annotations: t.annotations,
            })
            .collect())
    }

    /// Start a background task that listens for server-initiated notifications.
    fn start_notification_listener(&mut self) {
        let Some(mut rx) = self.transport.take_notification_receiver() else {
            return;
        };

        let dirty = Arc::clone(&self.tools_dirty);
        let server_id = self.config.id.clone();

        tokio::spawn(async move {
            while let Some(notification) = rx.recv().await {
                if notification.method == "notifications/tools/listChanged" {
                    dirty.store(true, Ordering::SeqCst);
                    tracing::debug!("MCP server `{server_id}` signaled tools/listChanged");
                }
            }
            tracing::debug!("MCP server `{server_id}` notification listener stopped");
        });
    }
}

#[async_trait]
impl McpServer for LiveMcpServer {
    fn server_id(&self) -> &str {
        &self.config.id
    }

    fn server_name(&self) -> &str {
        // Prefer server_info.name from initialize, then config name, then id
        if let Some(info) = &self.server_info {
            return &info.name;
        }
        self.config.name.as_deref().unwrap_or(&self.config.id)
    }

    fn version(&self) -> Option<&str> {
        self.server_info.as_ref().and_then(|i| i.version.as_deref())
    }

    fn supports_list_changed(&self) -> bool {
        self.server_capabilities
            .tools
            .as_ref()
            .is_some_and(|t| t.list_changed)
    }

    async fn tools(&self) -> McpResult<Vec<McpToolInfo>> {
        // If dirty, refetch and update cache. On failure, restore the dirty
        // flag so the next call retries instead of serving stale data.
        if self.tools_dirty.swap(false, Ordering::SeqCst) {
            match self.fetch_tools().await {
                Ok(tools) => {
                    let mut cache = self.tools_cache.write().await;
                    *cache = Some(tools.clone());
                    return Ok(tools);
                }
                Err(e) => {
                    self.tools_dirty.store(true, Ordering::SeqCst);
                    return Err(e);
                }
            }
        }

        // Return cached tools if available
        {
            let cache = self.tools_cache.read().await;
            if let Some(tools) = &*cache {
                return Ok(tools.clone());
            }
        }

        // First call — populate cache
        let tools = self.fetch_tools().await?;
        let mut cache = self.tools_cache.write().await;
        *cache = Some(tools.clone());
        Ok(tools)
    }

    async fn call_tool(
        &self,
        tool_name: &str,
        input: serde_json::Value,
    ) -> McpResult<McpToolResult> {
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": input,
        });

        let result = self
            .transport
            .request("tools/call", Some(params), self.tool_timeout)
            .await?;

        let call_result: ToolCallResult =
            serde_json::from_value(result).map_err(|e| McpError::Protocol {
                server_id: self.config.id.clone(),
                message: format!("invalid tools/call response: {e}"),
            })?;

        Ok(McpToolResult {
            content: call_result
                .content
                .into_iter()
                .map(content_block_to_content)
                .collect(),
            structured_content: call_result.structured_content,
            is_error: call_result.is_error,
        })
    }

    async fn refresh_tools(&self) -> McpResult<()> {
        let tools = self.fetch_tools().await?;
        let mut cache = self.tools_cache.write().await;
        *cache = Some(tools);
        self.tools_dirty.store(false, Ordering::SeqCst);
        Ok(())
    }
}

/// Convert a protocol-level content block to the trait-level content type.
fn content_block_to_content(block: McpContentBlock) -> McpContent {
    match block {
        McpContentBlock::Text { text } => McpContent::Text { text },
        McpContentBlock::Image { data, mime_type } => McpContent::Image { data, mime_type },
        McpContentBlock::Audio { data, mime_type } => McpContent::Audio { data, mime_type },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::config::McpServerConfig;

    /// Helper: create a `McpServerConfig` for a Python mock server script.
    fn mock_server_config(script: &str) -> McpServerConfig {
        McpServerConfig {
            id: "test-server".into(),
            name: None,
            transport: TransportConfig::Stdio {
                command: "python3".into(),
                args: vec!["-c".into(), script.into()],
            },
            env: HashMap::new(),
            enabled: true,
            source: None,
        }
    }

    /// A minimal MCP server that responds to initialize and tools/list.
    const MOCK_MCP_SERVER: &str = r#"
import sys, json

def respond(id, result):
    msg = json.dumps({"jsonrpc": "2.0", "id": id, "result": result})
    sys.stdout.write(msg + "\n")
    sys.stdout.flush()

for line in sys.stdin:
    req = json.loads(line.strip())
    method = req.get("method")
    req_id = req.get("id")

    if req_id is None:
        # notification, ignore
        continue

    if method == "initialize":
        respond(req_id, {
            "protocolVersion": "2025-03-26",
            "capabilities": {"tools": {"listChanged": False}},
            "serverInfo": {"name": "mock-server", "version": "0.1.0"}
        })
    elif method == "tools/list":
        respond(req_id, {
            "tools": [
                {
                    "name": "greet",
                    "description": "Say hello",
                    "inputSchema": {"type": "object", "properties": {"name": {"type": "string"}}}
                },
                {
                    "name": "add",
                    "description": "Add two numbers",
                    "inputSchema": {"type": "object", "properties": {"a": {"type": "number"}, "b": {"type": "number"}}}
                }
            ]
        })
    elif method == "tools/call":
        name = req["params"]["name"]
        args = req["params"].get("arguments", {})
        if name == "greet":
            respond(req_id, {
                "content": [{"type": "text", "text": f"Hello, {args.get('name', 'world')}!"}],
                "isError": False
            })
        elif name == "add":
            result = args.get("a", 0) + args.get("b", 0)
            respond(req_id, {
                "content": [{"type": "text", "text": str(result)}],
                "isError": False
            })
        else:
            respond(req_id, {
                "content": [{"type": "text", "text": f"Unknown tool: {name}"}],
                "isError": True
            })
    else:
        respond(req_id, {"error": {"code": -32601, "message": f"Method not found: {method}"}})
"#;

    /// MCP server that declares `listChanged: true` and sends the notification
    /// after `tools/list` is called twice.
    const MOCK_LIST_CHANGED_SERVER: &str = r#"
import sys, json

list_call_count = 0

def respond(id, result):
    msg = json.dumps({"jsonrpc": "2.0", "id": id, "result": result})
    sys.stdout.write(msg + "\n")
    sys.stdout.flush()

def send_notification(method):
    msg = json.dumps({"jsonrpc": "2.0", "method": method})
    sys.stdout.write(msg + "\n")
    sys.stdout.flush()

tools_v1 = [{"name": "old_tool", "description": "Original tool"}]
tools_v2 = [{"name": "new_tool", "description": "Updated tool"}]

for line in sys.stdin:
    req = json.loads(line.strip())
    method = req.get("method")
    req_id = req.get("id")

    if req_id is None:
        continue

    if method == "initialize":
        respond(req_id, {
            "protocolVersion": "2025-03-26",
            "capabilities": {"tools": {"listChanged": True}},
            "serverInfo": {"name": "mock-list-changed"}
        })
    elif method == "tools/list":
        list_call_count += 1
        if list_call_count <= 1:
            respond(req_id, {"tools": tools_v1})
            # After first list, signal that tools changed
            send_notification("notifications/tools/listChanged")
        else:
            respond(req_id, {"tools": tools_v2})
    else:
        respond(req_id, {"error": {"code": -32601, "message": "not found"}})
"#;

    #[tokio::test]
    async fn connect_and_get_server_info() -> McpResult<()> {
        let config = mock_server_config(MOCK_MCP_SERVER);
        let server = LiveMcpServer::connect(config).await?;

        assert_eq!(server.server_id(), "test-server");
        assert_eq!(server.server_name(), "mock-server");
        assert_eq!(server.version(), Some("0.1.0"));
        assert!(!server.supports_list_changed());
        assert!(server.is_connected());

        server.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn list_tools() -> McpResult<()> {
        let config = mock_server_config(MOCK_MCP_SERVER);
        let server = LiveMcpServer::connect(config).await?;

        let tools = server.tools().await?;
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "greet");
        assert_eq!(tools[1].name, "add");
        assert!(tools[0].description.as_deref() == Some("Say hello"));
        assert!(tools[0].input_schema.is_some());

        server.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn tools_are_cached() -> McpResult<()> {
        let config = mock_server_config(MOCK_MCP_SERVER);
        let server = LiveMcpServer::connect(config).await?;

        let tools1 = server.tools().await?;
        let tools2 = server.tools().await?;

        // Both calls should return the same tools (second from cache)
        assert_eq!(tools1.len(), tools2.len());
        assert_eq!(tools1[0].name, tools2[0].name);

        server.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn call_tool_text_result() -> McpResult<()> {
        let config = mock_server_config(MOCK_MCP_SERVER);
        let server = LiveMcpServer::connect(config).await?;

        let result = server
            .call_tool("greet", serde_json::json!({"name": "Alice"}))
            .await?;

        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);
        match &result.content[0] {
            McpContent::Text { text } => assert_eq!(text, "Hello, Alice!"),
            other => panic!("expected text content, got {other:?}"),
        }

        server.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn call_tool_computation() -> McpResult<()> {
        let config = mock_server_config(MOCK_MCP_SERVER);
        let server = LiveMcpServer::connect(config).await?;

        let result = server
            .call_tool("add", serde_json::json!({"a": 3, "b": 7}))
            .await?;

        assert!(!result.is_error);
        match &result.content[0] {
            McpContent::Text { text } => assert_eq!(text, "10"),
            other => panic!("expected text content, got {other:?}"),
        }

        server.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn call_unknown_tool_returns_error() -> McpResult<()> {
        let config = mock_server_config(MOCK_MCP_SERVER);
        let server = LiveMcpServer::connect(config).await?;

        let result = server
            .call_tool("nonexistent", serde_json::json!({}))
            .await?;

        assert!(result.is_error);

        server.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn refresh_tools_updates_cache() -> McpResult<()> {
        let config = mock_server_config(MOCK_MCP_SERVER);
        let server = LiveMcpServer::connect(config).await?;

        // Populate cache
        let tools1 = server.tools().await?;
        assert_eq!(tools1.len(), 2);

        // Explicit refresh
        server.refresh_tools().await?;

        // Cache should still be valid with refreshed data
        let tools2 = server.tools().await?;
        assert_eq!(tools2.len(), 2);

        server.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn tools_dirty_flag_triggers_refetch() -> McpResult<()> {
        let config = mock_server_config(MOCK_LIST_CHANGED_SERVER);
        let server = LiveMcpServer::connect(config).await?;

        assert!(server.supports_list_changed());

        // First tools() call gets v1
        let tools1 = server.tools().await?;
        assert_eq!(tools1.len(), 1);
        assert_eq!(tools1[0].name, "old_tool");

        // The server sends listChanged notification after the first tools/list.
        // Give the notification listener time to process it.
        tokio::time::sleep(Duration::from_millis(100)).await;

        // The dirty flag should now be set, causing a refetch
        let tools2 = server.tools().await?;
        assert_eq!(tools2.len(), 1);
        assert_eq!(tools2[0].name, "new_tool");

        server.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn server_name_falls_back_to_config() -> McpResult<()> {
        // Server that returns no serverInfo.name
        let script = r#"
import sys, json
for line in sys.stdin:
    req = json.loads(line.strip())
    if req.get("id") is None:
        continue
    if req["method"] == "initialize":
        msg = json.dumps({"jsonrpc": "2.0", "id": req["id"], "result": {
            "protocolVersion": "2025-03-26",
            "capabilities": {}
        }})
        sys.stdout.write(msg + "\n")
        sys.stdout.flush()
    else:
        msg = json.dumps({"jsonrpc": "2.0", "id": req["id"], "result": {"tools": []}})
        sys.stdout.write(msg + "\n")
        sys.stdout.flush()
"#;
        let mut config = mock_server_config(script);
        config.name = Some("My Custom Server".into());

        let server = LiveMcpServer::connect(config).await?;

        // No serverInfo → falls back to config name
        assert_eq!(server.server_name(), "My Custom Server");
        assert_eq!(server.version(), None);

        server.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn connect_failure_on_bad_command() {
        let config = McpServerConfig {
            id: "bad".into(),
            name: None,
            transport: TransportConfig::Stdio {
                command: "this-command-does-not-exist-12345".into(),
                args: vec![],
            },
            env: HashMap::new(),
            enabled: true,
            source: None,
        };

        let Err(err) = LiveMcpServer::connect(config).await else {
            panic!("expected ConnectionFailed error");
        };
        assert!(
            matches!(err, McpError::ConnectionFailed { .. }),
            "expected ConnectionFailed, got {err:?}"
        );
    }

    #[tokio::test]
    async fn http_transport_connect_failure() {
        // Connecting to a non-existent HTTP endpoint should fail during
        // the initialize handshake, not during transport creation.
        let config = McpServerConfig {
            id: "http-server".into(),
            name: None,
            transport: TransportConfig::Http {
                url: "http://127.0.0.1:1/mcp".into(),
                headers: HashMap::new(),
            },
            env: HashMap::new(),
            enabled: true,
            source: None,
        };

        let result = LiveMcpServer::connect(config).await;
        assert!(
            result.is_err(),
            "expected connection to fail for unreachable endpoint"
        );
    }
}
