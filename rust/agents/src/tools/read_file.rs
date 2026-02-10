//! `read_file` tool: read file contents with optional offset/limit (spec 3.3).

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::execution::FileContent;
use crate::registry::ToolExecutorFn;

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
// TODO(spec-ambiguity): The spec (line 500) says image data should be returned
// for multimodal models, and PLAN.md:199 expects `FileContent::Image` to be
// mapped to image content. However, `ToolExecutorFn` returns `String`, so raw
// image bytes cannot flow through without either base64 encoding or changing
// the return type to support multi-part content. Returning a placeholder string
// for now; proper image support requires a return type change (spec: 3.3).
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
                    FileContent::Text(text) => text,
                    FileContent::Image { media_type, .. } => {
                        format!("[Image file: {file_path} ({media_type})]")
                    }
                };

                Ok(output)
            })
        },
    )
}
