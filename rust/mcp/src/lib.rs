#![warn(clippy::pedantic)]

#[cfg(feature = "cli")]
pub mod cli;
pub mod config;
pub mod error;
pub mod pool;
pub mod server;
pub mod traits;
pub mod transport;
pub mod types;

pub use config::{ConfigSource, McpServerConfig, TransportConfig, discover};
pub use error::{McpError, McpResult, PrettyDuration};
pub use pool::ConnectionPool;
pub use server::LiveMcpServer;
pub use traits::{McpContent, McpServer, McpToolInfo, McpToolResult};
pub use transport::Transport;
pub use transport::http::HttpTransport;
pub use transport::stdio::StdioTransport;
pub use types::{
    InitializeResult, JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
    McpContentBlock, McpToolDefinition, ServerCapabilities, ServerInfo, ServerNotification,
    ToolCallResult, ToolsCapability, ToolsListResult,
};
