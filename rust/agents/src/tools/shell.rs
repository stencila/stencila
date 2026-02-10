//! `shell` tool: execute shell commands with timeout (spec 3.3).

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::registry::ToolExecutorFn;

use super::required_str;

/// Default timeout in milliseconds (10 seconds).
const DEFAULT_TIMEOUT_MS: u64 = 10_000;

/// Tool definition matching `tests/fixtures/tool_schemas/shell.json`.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "shell".into(),
        description: "Execute a shell command and return its output. The command runs in \
            a bash shell with a configurable timeout."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute."
                },
                "timeout_ms": {
                    "type": "integer",
                    "description": "Timeout in milliseconds. Defaults to 10000 (10 seconds).",
                    "minimum": 1
                },
                "description": {
                    "type": "string",
                    "description": "Human-readable description of what the command does (for logging only)."
                }
            },
            "required": ["command"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

/// Default maximum timeout in milliseconds (10 minutes).
const DEFAULT_MAX_TIMEOUT_MS: u64 = 600_000;

/// Executor with the default 10-second timeout and 10-minute max.
pub fn executor() -> ToolExecutorFn {
    executor_with_timeout(DEFAULT_TIMEOUT_MS, DEFAULT_MAX_TIMEOUT_MS)
}

/// Executor with a custom default timeout and maximum timeout.
///
/// Provider profiles use this to set profile-specific defaults (e.g. Anthropic 120s).
/// The `max_timeout_ms` parameter clamps any per-call `timeout_ms` value to prevent
/// unbounded execution time (spec `max_command_timeout_ms`).
pub fn executor_with_timeout(default_timeout_ms: u64, max_timeout_ms: u64) -> ToolExecutorFn {
    Box::new(
        move |args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move {
                let command = required_str(&args, "command")?;
                let timeout_ms = args
                    .get("timeout_ms")
                    .and_then(Value::as_u64)
                    .unwrap_or(default_timeout_ms)
                    .min(max_timeout_ms);

                let result = env.exec_command(command, timeout_ms, None, None).await?;

                let mut output = format!(
                    "Exit code: {}\nDuration: {}ms",
                    result.exit_code, result.duration_ms
                );

                if result.timed_out {
                    output.push_str("\n[TIMED OUT]");
                }

                if !result.stdout.is_empty() {
                    output.push_str("\n\nSTDOUT:\n");
                    output.push_str(&result.stdout);
                }

                if !result.stderr.is_empty() {
                    output.push_str("\n\nSTDERR:\n");
                    output.push_str(&result.stderr);
                }

                Ok(output)
            })
        },
    )
}
