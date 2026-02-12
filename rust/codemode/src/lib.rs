#![warn(clippy::pedantic)]

pub(crate) mod codegen;
pub mod error;
pub mod identifiers;
pub(crate) mod modules;
mod run;
pub mod sandbox;
pub(crate) mod search;
pub mod types;

pub use codegen::generate_declarations;
pub use error::{CodemodeError, LimitKind};
pub use identifiers::{
    normalize_server_id, resolve_export_collisions, resolve_server_collisions, tool_name_to_export,
};
pub use run::codemode_run;
pub use sandbox::Sandbox;
pub use stencila_mcp::{McpContent, McpServer, McpToolInfo, McpToolResult};
pub use types::{
    DetailLevel, Diagnostic, DiagnosticCode, DiagnosticSeverity, DirtyServerTracker, Limits,
    ListToolsOptions, LogEntry, LogLevel, RunRequest, RunResponse, SearchResultEntry,
    SearchResults, SearchToolsOptions, ServerDescription, ServerInfo, ToolDefinition, ToolSummary,
    ToolTraceEntry,
};
