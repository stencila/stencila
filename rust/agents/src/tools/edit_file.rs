//! `edit_file` tool: exact string replacement in files (spec 3.3).

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::error::AgentError;
use crate::execution::ExecutionEnvironment;
use crate::registry::{ToolExecutorFn, ToolOutput};

use super::required_str;

/// Tool definition matching `tests/fixtures/tool_schemas/edit_file.json`.
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "edit_file".into(),
        description: "Perform an exact string replacement in a file. The old_string must \
            match exactly (including whitespace and indentation). For multiple \
            replacements, set replace_all to true."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to edit."
                },
                "old_string": {
                    "type": "string",
                    "description": "The exact string to find and replace."
                },
                "new_string": {
                    "type": "string",
                    "description": "The replacement string."
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "If true, replace all occurrences. If false (default), the old_string must appear exactly once.",
                    "default": false
                }
            },
            "required": ["file_path", "old_string", "new_string"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

/// Executor that performs exact string replacement.
pub fn executor() -> ToolExecutorFn {
    Box::new(|args: Value, env: &dyn ExecutionEnvironment| {
        Box::pin(async move {
            let file_path = required_str(&args, "file_path")?;
            let old_string = required_str(&args, "old_string")?;
            let new_string = required_str(&args, "new_string")?;
            let replace_all = args
                .get("replace_all")
                .and_then(Value::as_bool)
                .unwrap_or(false);

            // Check file exists
            if !env.file_exists(file_path).await {
                return Err(AgentError::FileNotFound {
                    path: file_path.to_string(),
                });
            }

            // Read full file and strip line numbers
            let raw = super::read_raw_content(env, file_path).await?;

            // Count occurrences
            let count = raw.matches(old_string).count();

            if count == 0 {
                return Err(AgentError::EditConflict {
                    reason: format!("old_string not found in {file_path}"),
                });
            }

            if count > 1 && !replace_all {
                return Err(AgentError::EditConflict {
                    reason: format!(
                        "old_string found {count} times in {file_path}; \
                         use replace_all to replace all occurrences"
                    ),
                });
            }

            // Perform replacement
            let updated = if replace_all {
                raw.replace(old_string, new_string)
            } else {
                raw.replacen(old_string, new_string, 1)
            };

            // Write back
            env.write_file(file_path, &updated).await?;

            Ok(ToolOutput::Text(format!(
                "Replaced {count} occurrence(s) in {file_path}"
            )))
        })
    })
}
