use crate::error::CodemodeError;

/// Information about an MCP tool, abstracting over the underlying MCP protocol types.
#[derive(Debug, Clone)]
pub struct McpToolInfo {
    /// The canonical MCP tool name.
    pub name: String,

    /// A description of what the tool does.
    pub description: Option<String>,

    /// JSON Schema for the tool's input parameters.
    pub input_schema: Option<serde_json::Value>,

    /// JSON Schema for the tool's output.
    pub output_schema: Option<serde_json::Value>,

    /// Tool annotations (e.g. `readOnlyHint`, `destructiveHint`).
    pub annotations: Option<serde_json::Value>,
}

/// The result of calling an MCP tool.
#[derive(Debug, Clone)]
pub struct McpToolResult {
    /// Content blocks returned by the tool.
    pub content: Vec<McpContent>,

    /// Structured content (takes priority over `content` per §5.3.2 unwrapping).
    pub structured_content: Option<serde_json::Value>,

    /// Whether the tool returned an error result.
    pub is_error: bool,
}

/// A single content block from an MCP tool result.
#[derive(Debug, Clone)]
pub enum McpContent {
    /// Text content.
    Text { text: String },

    /// Base64-encoded image content.
    Image { data: String, mime_type: String },

    /// Base64-encoded audio content.
    Audio { data: String, mime_type: String },
}

/// An abstraction over an MCP server connection.
///
/// Implementations provide server metadata, tool discovery, and tool invocation.
/// Each server is identified by a unique `server_id` and exposes a set of tools.
#[async_trait::async_trait]
pub trait McpServer: Send + Sync {
    /// The unique identifier for this server.
    fn server_id(&self) -> &str;

    /// The human-readable name of this server.
    fn server_name(&self) -> &str;

    /// An optional description of what this server does.
    fn description(&self) -> Option<&str> {
        None
    }

    /// An optional version string for this server.
    fn version(&self) -> Option<&str> {
        None
    }

    /// Coarse capability categories this server provides.
    fn capabilities(&self) -> Option<Vec<String>> {
        None
    }

    /// List all tools available on this server.
    async fn tools(&self) -> Result<Vec<McpToolInfo>, CodemodeError>;

    /// Call a tool on this server with the given JSON input.
    async fn call_tool(
        &self,
        tool_name: &str,
        input: serde_json::Value,
    ) -> Result<McpToolResult, CodemodeError>;

    /// Whether this server supports `tools/listChanged` notifications.
    fn supports_list_changed(&self) -> bool {
        false
    }

    /// Refresh the server's tool definitions (called when tools have changed).
    ///
    /// The default implementation is a no-op.
    async fn refresh_tools(&self) -> Result<(), CodemodeError> {
        Ok(())
    }
}
