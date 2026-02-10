//! `glob` tool: find files by glob pattern (spec 3.3).

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::registry::{ToolExecutorFn, ToolOutput};

use super::required_str;

/// Tool definition matching `tests/fixtures/tool_schemas/glob.json`.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "glob".into(),
        description: "Find files matching a glob pattern. Returns file paths sorted by \
            modification time (newest first)."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern to match files (e.g., \"**/*.rs\", \"src/*.py\")."
                },
                "path": {
                    "type": "string",
                    "description": "Base directory for the search. Defaults to the working directory."
                }
            },
            "required": ["pattern"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

/// Executor that delegates to `env.glob_files()`.
pub fn executor() -> ToolExecutorFn {
    Box::new(
        |args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move {
                let pattern = required_str(&args, "pattern")?;
                let path = args
                    .get("path")
                    .and_then(Value::as_str)
                    .unwrap_or_else(|| env.working_directory());

                let files = env.glob_files(pattern, path).await?;

                if files.is_empty() {
                    Ok(ToolOutput::Text("No files found.".into()))
                } else {
                    Ok(ToolOutput::Text(files.join("\n")))
                }
            })
        },
    )
}
