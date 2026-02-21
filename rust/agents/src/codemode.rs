//! Codemode tool registration and MCP orchestration via JavaScript.
//!
//! When `enable_mcp_codemode` is active, a single `mcp_codemode` tool is registered.
//! The LLM writes JavaScript to orchestrate MCP calls in a sandboxed QuickJS
//! environment. TypeScript declarations are included in the system prompt so
//! the LLM knows what functions are available.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use serde_json::json;
use stencila_codemode::{
    DirtyServerTracker, RunRequest, RunResponse, codemode_run, generate_declarations,
};
use stencila_mcp::ConnectionPool;
use stencila_models3::types::tool::ToolDefinition;

use crate::error::AgentResult;
use crate::profile::ProviderProfile;
use crate::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};

/// Tool name for the MCP codemode tool.
pub const TOOL_CODEMODE: &str = "mcp_codemode";

/// Maximum character budget for TypeScript declarations in the system prompt.
const DECLARATION_BUDGET: usize = 4000;

// ---------------------------------------------------------------------------
// Tool registration
// ---------------------------------------------------------------------------

/// Register the `mcp_codemode` tool in the profile's tool registry.
///
/// The tool takes `code` (JavaScript), optional `timeout_ms`, and optional
/// `max_tool_calls` parameters. The executor runs the code in a sandboxed
/// QuickJS environment with access to MCP servers.
///
/// When `allowed` is `Some`, only the listed MCP server IDs are visible to
/// codemode at runtime.
pub fn register_codemode_tool(
    profile: &mut dyn ProviderProfile,
    pool: &Arc<ConnectionPool>,
    dirty_tracker: &Arc<Mutex<DirtyServerTracker>>,
    allowed: Option<Vec<String>>,
) -> AgentResult<()> {
    let tool = RegisteredTool::new(definition(), executor(pool, dirty_tracker, allowed));
    profile.tool_registry_mut().register(tool)
}

/// Tool definition for `codemode`.
fn definition() -> ToolDefinition {
    ToolDefinition {
        name: TOOL_CODEMODE.into(),
        description: "Execute JavaScript code in a sandboxed environment with access to \
                       MCP server tools. Use the TypeScript declarations in the system \
                       prompt to discover available functions. Import tools from \
                       `@codemode/servers/<server_id>` modules."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "JavaScript code to execute. Use ES module syntax. \
                                    Set `globalThis.__codemode_result__` or use \
                                    `export default` to return a value."
                },
                "timeout_ms": {
                    "type": "integer",
                    "description": "Execution timeout in milliseconds (default: 30000)."
                },
                "max_tool_calls": {
                    "type": "integer",
                    "description": "Maximum number of MCP tool calls allowed (default: 50)."
                }
            },
            "required": ["code"]
        }),
        strict: false,
    }
}

/// Build an async executor for the `codemode` tool.
fn executor(
    pool: &Arc<ConnectionPool>,
    dirty_tracker: &Arc<Mutex<DirtyServerTracker>>,
    allowed: Option<Vec<String>>,
) -> ToolExecutorFn {
    let pool = Arc::clone(pool);
    let tracker = Arc::clone(dirty_tracker);

    Box::new(move |args, _env| {
        let pool = Arc::clone(&pool);
        let tracker = Arc::clone(&tracker);
        let allowed = allowed.clone();
        Box::pin(async move {
            let code = crate::tools::required_str(&args, "code")?.to_string();

            let timeout_ms = args.get("timeout_ms").and_then(serde_json::Value::as_u64);
            let max_tool_calls = args
                .get("max_tool_calls")
                .and_then(serde_json::Value::as_u64)
                .and_then(|v| u32::try_from(v).ok());

            let limits = if timeout_ms.is_some() || max_tool_calls.is_some() {
                Some(stencila_codemode::Limits {
                    timeout_ms,
                    max_memory_bytes: None,
                    max_log_bytes: None,
                    max_tool_calls,
                })
            } else {
                None
            };

            let request = RunRequest {
                code,
                limits,
                requested_capabilities: None,
            };

            // Gather connected servers, filtered by allow-list
            let servers = crate::mcp::filter_servers(&pool, allowed.as_deref()).await;

            // Take dirty server set
            let dirty: HashSet<String> = tracker
                .lock()
                .map(|mut t| t.take_dirty())
                .unwrap_or_default();

            let response = codemode_run(&request, &servers, &dirty).await;

            Ok(ToolOutput::Text(format_codemode_response(&response)))
        })
    })
}

// ---------------------------------------------------------------------------
// System prompt
// ---------------------------------------------------------------------------

/// Build the codemode section of the system prompt.
///
/// Always includes a listing of available MCP servers with their
/// instructions/descriptions. When TypeScript declarations fit within the
/// character budget they are appended; otherwise a hint to use runtime
/// discovery is included.
///
/// When `allowed` is `Some`, only those server IDs are included.
pub async fn build_codemode_prompt(
    pool: &Arc<ConnectionPool>,
    allowed: Option<&[String]>,
) -> String {
    let servers = crate::mcp::filter_servers(pool, allowed).await;

    if servers.is_empty() {
        return String::new();
    }

    let mut prompt = format!(
        "# MCP Codemode\n\n\
         Use the `{TOOL_CODEMODE}` tool to execute JavaScript with access to MCP servers.\n\n",
    );

    // Always list servers with instructions/descriptions
    prompt.push_str("## Available MCP servers\n\n");
    for server in &servers {
        let server_id = server.server_id();
        let server_name = server.server_name();
        let tool_count = server.tools().await.map(|t| t.len()).unwrap_or(0);

        prompt.push_str(&format!(
            "- **{server_name}** (`{server_id}`): {tool_count} tools\n"
        ));

        if let Some(instructions) = server.instructions() {
            for line in instructions.lines() {
                prompt.push_str(&format!("  {line}\n"));
            }
        } else if let Some(description) = server.description() {
            prompt.push_str(&format!("  {description}\n"));
        }
    }

    // Try to generate full TypeScript declarations
    match generate_declarations(&servers).await {
        Ok(declarations) if declarations.len() <= DECLARATION_BUDGET => {
            prompt.push_str(&format!(
                "\n## TypeScript declarations\n\n\
                 ```typescript\n{declarations}\n```\n"
            ));
        }
        Ok(_) | Err(_) => {
            prompt.push_str(
                "\nThe full TypeScript declarations are too large for the system prompt. \
                 Use `import { listServers, listTools } from '@codemode/discovery'` \
                 to explore available tools at runtime.\n",
            );
        }
    }

    prompt
}

// ---------------------------------------------------------------------------
// Response formatting
// ---------------------------------------------------------------------------

/// Format a [`RunResponse`] as text for the LLM.
///
/// Includes the result, logs, diagnostics, and tool trace in a structured
/// format that is easy for the LLM to parse.
#[must_use]
pub fn format_codemode_response(response: &RunResponse) -> String {
    let mut sections: Vec<String> = Vec::new();

    // Result
    if !response.result.is_null() {
        let result_str = if response.result.is_string() {
            response.result.as_str().unwrap_or("").to_string()
        } else {
            serde_json::to_string_pretty(&response.result).unwrap_or_default()
        };
        sections.push(format!("Result:\n{result_str}"));
    }

    // Diagnostics (errors, warnings)
    if !response.diagnostics.is_empty() {
        let diag_lines: Vec<String> = response
            .diagnostics
            .iter()
            .map(|d| format!("[{:?}] {:?}: {}", d.severity, d.code, d.message))
            .collect();
        sections.push(format!("Diagnostics:\n{}", diag_lines.join("\n")));
    }

    // Logs
    if !response.logs.is_empty() {
        let log_lines: Vec<String> = response
            .logs
            .iter()
            .map(|l| format!("[{:?}] {}", l.level, l.message))
            .collect();
        sections.push(format!("Logs:\n{}", log_lines.join("\n")));
    }

    // Tool trace
    if let Some(ref trace) = response.tool_trace
        && !trace.is_empty()
    {
        let trace_lines: Vec<String> = trace
            .iter()
            .map(|t| {
                let status = if t.ok { "ok" } else { "error" };
                format!(
                    "  {}.{} ({}ms) → {status}",
                    t.server_id, t.tool_name, t.duration_ms
                )
            })
            .collect();
        sections.push(format!("Tool calls:\n{}", trace_lines.join("\n")));
    }

    if sections.is_empty() {
        "(no output)".to_string()
    } else {
        sections.join("\n\n")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use stencila_codemode::{
        Diagnostic, DiagnosticCode, DiagnosticSeverity, LogEntry, LogLevel, ToolTraceEntry,
    };

    use super::*;

    #[test]
    fn definition_has_required_fields() {
        let def = definition();
        assert_eq!(def.name, TOOL_CODEMODE);
        assert!(!def.description.is_empty());

        let required = def.parameters.get("required").and_then(|v| v.as_array());
        assert!(required.is_some_and(|arr| arr.iter().any(|v| v.as_str() == Some("code"))));
    }

    #[test]
    fn format_response_with_result() {
        let response = RunResponse {
            result: serde_json::json!("hello world"),
            logs: vec![],
            diagnostics: vec![],
            tool_trace: None,
        };
        let output = format_codemode_response(&response);
        assert!(output.contains("Result:"));
        assert!(output.contains("hello world"));
    }

    #[test]
    fn format_response_with_diagnostics() {
        let response = RunResponse {
            result: serde_json::Value::Null,
            logs: vec![],
            diagnostics: vec![Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode::UncaughtException,
                message: "TypeError: undefined is not a function".into(),
                hint: None,
                path: None,
                error_class: None,
            }],
            tool_trace: None,
        };
        let output = format_codemode_response(&response);
        assert!(output.contains("Diagnostics:"));
        assert!(output.contains("TypeError"));
    }

    #[test]
    fn format_response_with_logs() {
        let response = RunResponse {
            result: serde_json::Value::Null,
            logs: vec![LogEntry {
                level: LogLevel::Log,
                message: "fetching data...".into(),
                time_ms: 42,
            }],
            diagnostics: vec![],
            tool_trace: None,
        };
        let output = format_codemode_response(&response);
        assert!(output.contains("Logs:"));
        assert!(output.contains("fetching data..."));
    }

    #[test]
    fn format_response_with_tool_trace() {
        let response = RunResponse {
            result: serde_json::Value::Null,
            logs: vec![],
            diagnostics: vec![],
            tool_trace: Some(vec![ToolTraceEntry {
                server_id: "fs-server".into(),
                tool_name: "read_file".into(),
                duration_ms: 15,
                ok: true,
                error: None,
            }]),
        };
        let output = format_codemode_response(&response);
        assert!(output.contains("Tool calls:"));
        assert!(output.contains("fs-server.read_file"));
        assert!(output.contains("15ms"));
        assert!(output.contains("ok"));
    }

    #[test]
    fn format_response_empty() {
        let response = RunResponse::default();
        let output = format_codemode_response(&response);
        assert_eq!(output, "(no output)");
    }

    #[test]
    fn format_response_json_result() {
        let response = RunResponse {
            result: serde_json::json!({"count": 5, "items": ["a", "b"]}),
            logs: vec![],
            diagnostics: vec![],
            tool_trace: None,
        };
        let output = format_codemode_response(&response);
        assert!(output.contains("Result:"));
        assert!(output.contains("\"count\": 5"));
    }

    #[test]
    fn format_response_failed_tool_trace() {
        let response = RunResponse {
            result: serde_json::Value::Null,
            logs: vec![],
            diagnostics: vec![],
            tool_trace: Some(vec![ToolTraceEntry {
                server_id: "api".into(),
                tool_name: "query".into(),
                duration_ms: 500,
                ok: false,
                error: Some("timeout".into()),
            }]),
        };
        let output = format_codemode_response(&response);
        assert!(output.contains("error"));
    }
}
