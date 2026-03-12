//! `write_design` tool: persist a software design specification.

use std::path::Path;

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::error::AgentError;
use crate::registry::{ToolExecutorFn, ToolOutput};

use super::required_str;

/// Filename for the ordered index of design names.
pub(crate) const INDEX_FILE: &str = "index.json";

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "write_design".into(),
        description: "Persist a software design specification as a Markdown file in \
            .stencila/designs/. Accepts a kebab-case design name and the full Markdown \
            content. Returns the path of the written file."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Kebab-case design name (e.g. 'user-auth-flow')."
                },
                "content": {
                    "type": "string",
                    "description": "Full Markdown content of the design specification."
                }
            },
            "required": ["name", "content"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

pub fn executor() -> ToolExecutorFn {
    Box::new(
        |args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move {
                let name = required_str(&args, "name")?;
                let content = required_str(&args, "content")?;

                let cwd = Path::new(env.working_directory());
                let stencila_dir = stencila_dirs::closest_stencila_dir(cwd, true)
                    .await
                    .map_err(|e| AgentError::Io {
                        message: format!("failed to find .stencila directory: {e}"),
                    })?;
                let designs_dir = stencila_dir.join("designs");

                // Ensure the designs directory exists
                tokio::fs::create_dir_all(&designs_dir)
                    .await
                    .map_err(|e| AgentError::Io {
                        message: format!(
                            "failed to create designs directory {}: {e}",
                            designs_dir.display()
                        ),
                    })?;

                let filename = format!("{name}.md");
                let file_path = designs_dir.join(&filename);

                tokio::fs::write(&file_path, content)
                    .await
                    .map_err(|e| AgentError::Io {
                        message: format!(
                            "failed to write design file {}: {e}",
                            file_path.display()
                        ),
                    })?;

                // Update the index: append name or move it to the end if it exists
                update_index(&designs_dir, name).await?;

                let path_str = file_path.display().to_string();
                Ok(ToolOutput::Text(format!("Design written to {path_str}")))
            })
        },
    )
}

/// Read the index array from `index.json`, returning an empty vec if missing or corrupt.
pub(crate) async fn read_index(designs_dir: &Path) -> Vec<String> {
    let index_path = designs_dir.join(INDEX_FILE);
    let Ok(data) = tokio::fs::read_to_string(&index_path).await else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<String>>(&data).unwrap_or_default()
}

/// Append `name` to the index (or move it to the end if already present) and persist.
async fn update_index(designs_dir: &Path, name: &str) -> Result<(), AgentError> {
    let mut index = read_index(designs_dir).await;

    // Remove existing entry so it moves to the end
    index.retain(|n| n != name);
    index.push(name.to_string());

    let index_path = designs_dir.join(INDEX_FILE);
    let serialized = serde_json::to_string_pretty(&index).map_err(|e| AgentError::Io {
        message: format!("failed to serialize design index: {e}"),
    })?;

    tokio::fs::write(&index_path, serialized)
        .await
        .map_err(|e| AgentError::Io {
            message: format!("failed to write design index {}: {e}", index_path.display()),
        })?;

    Ok(())
}
