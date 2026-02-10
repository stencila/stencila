//! `list_dir` tool: directory listing with optional depth (spec 3.6, Gemini-specific).

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::registry::ToolExecutorFn;

use super::required_str;

/// Tool definition matching `tests/fixtures/tool_schemas/list_dir.json`.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_dir".into(),
        description: "List the contents of a directory. Directories are shown with a \
            trailing slash; files include their size."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute path to the directory to list."
                },
                "depth": {
                    "type": "integer",
                    "description": "Maximum depth to recurse. Defaults to 1 (immediate children only).",
                    "minimum": 1
                }
            },
            "required": ["path"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

/// Executor that delegates to `env.list_directory()`.
pub fn executor() -> ToolExecutorFn {
    Box::new(
        |args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move {
                let path = required_str(&args, "path")?;
                let depth = args
                    .get("depth")
                    .and_then(Value::as_u64)
                    .map_or(1, |v| v as usize);

                let entries = env.list_directory(path, depth).await?;

                let lines: Vec<String> = entries
                    .iter()
                    .map(|entry| {
                        if entry.is_dir {
                            format!("{}/", entry.name)
                        } else {
                            match entry.size {
                                Some(size) => format!("{} ({size} bytes)", entry.name),
                                None => entry.name.clone(),
                            }
                        }
                    })
                    .collect();

                if lines.is_empty() {
                    Ok("Empty directory.".into())
                } else {
                    Ok(lines.join("\n"))
                }
            })
        },
    )
}
