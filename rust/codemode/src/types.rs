use serde::{Deserialize, Serialize};

/// Request to execute JavaScript code in the codemode sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRequest {
    /// JavaScript source text to execute (ES module semantics).
    pub code: String,

    /// Host-defined execution limits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<Limits>,

    /// Coarse capability hints for the execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_capabilities: Option<Vec<String>>,
}

/// Response from a codemode execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunResponse {
    /// Captured console output.
    pub logs: Vec<LogEntry>,

    /// The value of `globalThis.__codemode_result__`, or `null`.
    pub result: serde_json::Value,

    /// Structured diagnostics (errors, warnings, info).
    pub diagnostics: Vec<Diagnostic>,

    /// Redacted tool-call trace (no inputs/outputs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_trace: Option<Vec<ToolTraceEntry>>,
}

impl Default for RunResponse {
    fn default() -> Self {
        Self {
            logs: Vec::new(),
            result: serde_json::Value::Null,
            diagnostics: Vec::new(),
            tool_trace: None,
        }
    }
}

/// Execution limits for the sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    /// Maximum execution time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,

    /// Maximum memory usage in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_memory_bytes: Option<u64>,

    /// Maximum total bytes of console log output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_log_bytes: Option<u64>,

    /// Maximum number of MCP tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u32>,
}

/// A captured console log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    /// The severity level of the log.
    pub level: LogLevel,

    /// The concatenated, serialized message.
    pub message: String,

    /// Milliseconds since sandbox start.
    pub time_ms: u64,
}

/// Console log severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Log,
    Warn,
    Error,
}

/// A structured diagnostic from execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    /// Severity of the diagnostic.
    pub severity: DiagnosticSeverity,

    /// Machine-readable error code.
    pub code: DiagnosticCode,

    /// Human-readable description.
    pub message: String,

    /// Recommended corrective action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,

    /// Source location or JSON Pointer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// The `@codemode/errors` class name (e.g. `SchemaValidationError`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_class: Option<String>,
}

/// Diagnostic severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

/// Machine-readable diagnostic codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticCode {
    /// JavaScript syntax error in the provided code.
    #[serde(rename = "SYNTAX_ERROR")]
    SyntaxError,

    /// An uncaught exception during execution.
    #[serde(rename = "UNCAUGHT_EXCEPTION")]
    UncaughtException,

    /// A module import failed.
    #[serde(rename = "IMPORT_FAILURE")]
    ImportFailure,

    /// A sandbox limit was exceeded.
    #[serde(rename = "SANDBOX_LIMIT")]
    SandboxLimit,
}

/// A redacted tool-call trace entry (no inputs or outputs per spec).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolTraceEntry {
    /// The server that handled the call.
    pub server_id: String,

    /// The canonical MCP tool name.
    pub tool_name: String,

    /// Call duration in milliseconds.
    pub duration_ms: u64,

    /// Whether the call succeeded.
    pub ok: bool,

    /// Error summary, if the call failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// The level of detail to include in discovery results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DetailLevel {
    /// Only `toolName` and `exportName`.
    Name,
    /// Adds `description` and `annotations`.
    Description,
    /// Adds `inputSchema` and `outputSchema`.
    Full,
}

impl Default for DetailLevel {
    fn default() -> Self {
        Self::Description
    }
}

/// Basic information about a connected MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    /// The normalized server identifier.
    pub server_id: String,

    /// Human-readable server name.
    pub server_name: String,

    /// Coarse capability categories the server provides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<String>>,
}

/// Extended server information including description and version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerDescription {
    /// The normalized server identifier.
    pub server_id: String,

    /// Human-readable server name.
    pub server_name: String,

    /// Coarse capability categories the server provides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<String>>,

    /// A description of what the server does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The server version string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// A summary of an MCP tool (name-level or description-level detail).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSummary {
    /// The canonical MCP tool name.
    pub tool_name: String,

    /// The JS export identifier for this tool.
    pub export_name: String,

    /// A description of what the tool does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// MCP tool annotations (e.g. `readOnlyHint`, `destructiveHint`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<serde_json::Value>,
}

/// Full definition of an MCP tool including schemas.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    /// The canonical MCP tool name.
    pub tool_name: String,

    /// The JS export identifier for this tool.
    pub export_name: String,

    /// A description of what the tool does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// MCP tool annotations (e.g. `readOnlyHint`, `destructiveHint`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<serde_json::Value>,

    /// JSON Schema for the tool's input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,

    /// JSON Schema for the tool's output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
}

/// Results from a tool search query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    /// The original search query.
    pub query: String,

    /// Matching tools, each annotated with their `serverId`.
    pub results: Vec<SearchResultEntry>,
}

/// A single search result entry: a tool definition extended with `serverId`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultEntry {
    /// The server this tool belongs to.
    pub server_id: String,

    /// The canonical MCP tool name.
    pub tool_name: String,

    /// The JS export identifier for this tool.
    pub export_name: String,

    /// A description of what the tool does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// MCP tool annotations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<serde_json::Value>,

    /// JSON Schema for the tool's input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,

    /// JSON Schema for the tool's output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
}

/// Options for `listTools()`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListToolsOptions {
    /// Detail level for returned tool info. Defaults to `description`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<DetailLevel>,
}

/// Options for `searchTools()`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchToolsOptions {
    /// Detail level for returned tool info. Defaults to `description`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<DetailLevel>,

    /// Restrict search to a single server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,

    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}
