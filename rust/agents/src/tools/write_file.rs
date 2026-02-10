//! `write_file` tool: write content to a file (spec 3.3).

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::registry::ToolExecutorFn;

use super::required_str;

/// Tool definition matching `tests/fixtures/tool_schemas/write_file.json`.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "write_file".into(),
        description: "Write content to a file, creating it and any parent directories if \
            they do not exist. Overwrites the file if it already exists."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to write."
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file."
                }
            },
            "required": ["file_path", "content"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

/// Executor that delegates to `env.write_file()`.
pub fn executor() -> ToolExecutorFn {
    Box::new(
        |args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move {
                let file_path = required_str(&args, "file_path")?;
                let content = required_str(&args, "content")?;

                env.write_file(file_path, content).await?;

                let n = content.len();
                Ok(format!("Successfully wrote {n} bytes to {file_path}"))
            })
        },
    )
}
