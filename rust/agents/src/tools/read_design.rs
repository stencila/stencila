//! `read_design` tool: retrieve a persisted software design specification.

use std::path::Path;

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::error::AgentError;
use crate::registry::{ToolExecutorFn, ToolOutput};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "read_design".into(),
        description: "Read a persisted software design specification from .stencila/designs/. \
            When called with no arguments, returns the most recently written design. \
            When called with a name, returns that specific design."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Optional design name. Omit to read the latest design."
                }
            },
            "required": [],
            "additionalProperties": false
        }),
        strict: false,
    }
}

pub fn executor() -> ToolExecutorFn {
    Box::new(
        |args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move {
                let cwd = Path::new(env.working_directory());
                let stencila_dir = stencila_dirs::closest_stencila_dir(cwd, false)
                    .await
                    .map_err(|e| AgentError::Io {
                        message: format!("failed to find .stencila directory: {e}"),
                    })?;
                let designs_dir = stencila_dir.join("designs");

                if !designs_dir.exists() {
                    return Err(AgentError::ValidationError {
                        reason: "no designs directory found (.stencila/designs/)".into(),
                    });
                }

                let selector = args
                    .get("name")
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty());

                let name = match selector {
                    Some(name) => name.to_string(),
                    None => find_latest_name(&designs_dir).await?,
                };

                let file_path = designs_dir.join(format!("{name}.md"));

                let content =
                    tokio::fs::read_to_string(&file_path)
                        .await
                        .map_err(|e| AgentError::Io {
                            message: format!(
                                "failed to read design file {}: {e}",
                                file_path.display()
                            ),
                        })?;

                let path_str = file_path.display().to_string();
                Ok(ToolOutput::Text(format!("Design: {path_str}\n\n{content}")))
            })
        },
    )
}

/// Find the name of the most recently written design.
///
/// Reads the index first; falls back to filesystem modification time
/// if the index is missing or empty.
async fn find_latest_name(designs_dir: &Path) -> Result<String, AgentError> {
    let index = super::write_design::read_index(designs_dir).await;

    if let Some(last) = index.last() {
        return Ok(last.clone());
    }

    // Fallback: find the most recently modified .md file
    let entries = std::fs::read_dir(designs_dir).map_err(|e| AgentError::Io {
        message: format!(
            "failed to read designs directory {}: {e}",
            designs_dir.display()
        ),
    })?;

    let mut newest: Option<(std::path::PathBuf, std::time::SystemTime)> = None;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file()
            || path.extension().and_then(|e| e.to_str()) != Some("md")
            || path.file_name().and_then(|f| f.to_str()) == Some(super::write_design::INDEX_FILE)
        {
            continue;
        }
        if let Ok(meta) = path.metadata()
            && let Ok(modified) = meta.modified()
            && newest.as_ref().is_none_or(|(_, t)| modified > *t)
        {
            newest = Some((path, modified));
        }
    }

    let (path, _) = newest.ok_or_else(|| AgentError::ValidationError {
        reason: "no design files found in .stencila/designs/".into(),
    })?;

    // Extract the name by stripping the .md extension
    let name =
        path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| AgentError::ValidationError {
                reason: "invalid design filename".into(),
            })?;

    Ok(name.to_string())
}
