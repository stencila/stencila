//! `grep` tool: regex search across files (spec 3.3).

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::registry::{ToolExecutorFn, ToolOutput};
use crate::types::GrepOptions;

use super::required_str;

/// Tool definition matching `tests/fixtures/tool_schemas/grep.json`.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "grep".into(),
        description: "Search file contents using a regular expression pattern. Returns \
            matching lines with file paths and line numbers."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regular expression pattern to search for."
                },
                "path": {
                    "type": "string",
                    "description": "Directory or file to search in. Defaults to the working directory."
                },
                "glob_filter": {
                    "type": "string",
                    "description": "Glob pattern to filter files (e.g., \"*.rs\", \"*.py\")."
                },
                "case_insensitive": {
                    "type": "boolean",
                    "description": "If true, perform case-insensitive matching.",
                    "default": false
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of matching lines to return. Defaults to 100.",
                    "minimum": 1
                }
            },
            "required": ["pattern"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

/// Executor that delegates to `env.grep()`.
pub fn executor() -> ToolExecutorFn {
    Box::new(
        |args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move {
                let pattern = required_str(&args, "pattern")?;
                let path = args
                    .get("path")
                    .and_then(Value::as_str)
                    .unwrap_or_else(|| env.working_directory());

                let options = GrepOptions {
                    glob_filter: args
                        .get("glob_filter")
                        .and_then(Value::as_str)
                        .map(String::from),
                    case_insensitive: args
                        .get("case_insensitive")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    max_results: args
                        .get("max_results")
                        .and_then(Value::as_u64)
                        .map_or(100, |v| v as u32),
                };

                env.grep(pattern, path, &options)
                    .await
                    .map(ToolOutput::Text)
            })
        },
    )
}
