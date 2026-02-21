//! `read_file` tool: read file contents with optional offset/limit (spec 3.3).

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::execution::FileContent;
use crate::registry::{MAX_IMAGE_BYTES, ToolExecutorFn, ToolOutput};

use super::required_str;

/// Tool definition matching `tests/fixtures/tool_schemas/read_file.json`.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "read_file".into(),
        description: "Read the contents of a file. Output includes line numbers formatted \
            as `{line_number} | {content}`. Use offset and limit to read specific \
            portions of large files."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to read."
                },
                "offset": {
                    "type": "integer",
                    "description": "Line number to start reading from (1-based). Defaults to 1.",
                    "minimum": 1
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to read. Defaults to 2000.",
                    "minimum": 1
                }
            },
            "required": ["file_path"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

/// Executor that delegates to `env.read_file()`.
///
/// For image files, returns `ToolOutput::ImageWithText` carrying the raw
/// bytes alongside a text placeholder. Images larger than [`MAX_IMAGE_BYTES`]
/// fall back to `ToolOutput::Text` with just the placeholder to avoid
/// inflating memory and request payloads. (spec: 3.3, line 500)
pub fn executor() -> ToolExecutorFn {
    Box::new(
        |args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move {
                let file_path = required_str(&args, "file_path")?;
                let offset = args
                    .get("offset")
                    .and_then(Value::as_u64)
                    .map(|v| v as usize);
                let limit = args
                    .get("limit")
                    .and_then(Value::as_u64)
                    .map(|v| v as usize);

                let content = env.read_file(file_path, offset, limit).await?;

                let output = match content {
                    FileContent::Text(text) => ToolOutput::Text(text),
                    FileContent::Image { data, media_type } => {
                        let text = format!("[Image file: {file_path} ({media_type})]");
                        if data.len() <= MAX_IMAGE_BYTES {
                            ToolOutput::ImageWithText {
                                text,
                                data,
                                media_type,
                            }
                        } else {
                            ToolOutput::Text(text)
                        }
                    }
                };

                Ok(output)
            })
        },
    )
}
