use std::fmt;

/// The kind of sandbox limit that was exceeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitKind {
    /// Execution exceeded the configured timeout.
    Timeout,
    /// Memory usage exceeded the configured maximum.
    Memory,
    /// Console log output exceeded the configured byte limit.
    LogBytes,
    /// Number of tool calls exceeded the configured maximum.
    ToolCalls,
}

impl fmt::Display for LimitKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => write!(f, "timeout"),
            Self::Memory => write!(f, "memory"),
            Self::LogBytes => write!(f, "log bytes"),
            Self::ToolCalls => write!(f, "tool calls"),
        }
    }
}

/// Errors that can occur during codemode execution.
#[derive(Debug, thiserror::Error)]
pub enum CodemodeError {
    /// A runtime error occurred during JavaScript execution.
    #[error("runtime error: {0}")]
    Runtime(String),

    /// Input failed JSON Schema validation.
    #[error("schema validation error for tool `{tool_name}`: {message}")]
    SchemaValidation {
        /// The canonical MCP tool name.
        tool_name: String,
        /// The JS export name for the tool.
        export_name: String,
        /// Human-readable validation error message.
        message: String,
        /// JSON Pointer path to the failing property.
        path: Option<String>,
        /// A hint recommending corrective action.
        hint: Option<String>,
    },

    /// The requested tool does not exist on the server.
    #[error("tool `{tool_name}` not found on server `{server_id}`")]
    ToolNotFound {
        server_id: String,
        tool_name: String,
    },

    /// The requested server is not connected.
    #[error("server `{server_id}` not found")]
    ServerNotFound { server_id: String },

    /// A server ID contains no valid characters and cannot be normalized.
    #[error("invalid server ID `{server_id}`: normalizes to empty string")]
    InvalidServerId { server_id: String },

    /// An MCP tool call returned an error result (`isError: true`).
    #[error("tool call error for `{tool_name}` on server `{server_id}`: {message}")]
    ToolCallError {
        server_id: String,
        tool_name: String,
        message: String,
    },

    /// The MCP server rejected the call due to invalid or missing credentials.
    #[error("authentication error for server `{server_id}`: {message}")]
    Authentication { server_id: String, message: String },

    /// A sandbox limit was exceeded.
    #[error("sandbox limit exceeded: {kind}")]
    SandboxLimit { kind: LimitKind },

    /// An error from the `QuickJS` runtime.
    #[error("quickjs error: {0}")]
    QuickJs(String),

    /// An error propagated from the MCP crate.
    #[error("mcp error: {0}")]
    Mcp(#[from] stencila_mcp::McpError),
}
