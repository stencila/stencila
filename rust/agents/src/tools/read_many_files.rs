//! `read_many_files` tool: batch file reading (spec 3.6, Gemini-specific).

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::error::{AgentError, AgentResult};
use crate::execution::FileContent;
use crate::registry::ToolExecutorFn;

/// Tool definition matching `tests/fixtures/tool_schemas/read_many_files.json`.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "read_many_files".into(),
        description: "Read multiple files in a single call. Returns contents of each file \
            with headers. Errors for individual files are reported inline without \
            aborting the batch."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "paths": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Array of absolute file paths to read."
                }
            },
            "required": ["paths"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

/// Extract the `paths` array from JSON arguments.
fn extract_paths(args: &Value) -> AgentResult<Vec<&str>> {
    let arr =
        args.get("paths")
            .and_then(Value::as_array)
            .ok_or_else(|| AgentError::ValidationError {
                reason: "missing required array parameter: paths".into(),
            })?;

    arr.iter()
        .map(|v| {
            v.as_str().ok_or_else(|| AgentError::ValidationError {
                reason: "paths array must contain only strings".into(),
            })
        })
        .collect()
}

/// Executor that reads multiple files, reporting per-file errors inline.
pub fn executor() -> ToolExecutorFn {
    Box::new(
        |args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move {
                let paths = extract_paths(&args)?;
                let mut parts = Vec::with_capacity(paths.len());

                for path in paths {
                    let header = format!("=== {path} ===");
                    match env.read_file(path, None, None).await {
                        Ok(FileContent::Text(text)) => {
                            parts.push(format!("{header}\n{text}"));
                        }
                        Ok(FileContent::Image { media_type, .. }) => {
                            parts.push(format!("{header}\n[Image file: {path} ({media_type})]"));
                        }
                        Err(e) => {
                            parts.push(format!("{header}\n[Error: {e}]"));
                        }
                    }
                }

                Ok(parts.join("\n"))
            })
        },
    )
}
