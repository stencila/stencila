//! `read_plan` tool: retrieve a persisted implementation plan.

use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::error::AgentError;
use crate::registry::{ToolExecutorFn, ToolOutput};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "read_plan".into(),
        description: "Read a persisted implementation plan from .stencila/plans/. \
            When called with no arguments, returns the most recently written plan. \
            When called with a name, returns that specific plan."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Optional plan name. Omit to read the latest plan."
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
                let plans_dir = stencila_dir.join("plans");

                if !plans_dir.exists() {
                    return Err(AgentError::ValidationError {
                        reason: "no plans directory found (.stencila/plans/)".into(),
                    });
                }

                let selector = args
                    .get("name")
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty());

                let name = match selector {
                    Some(name) => name.to_string(),
                    None => find_latest_name(&plans_dir)?,
                };

                let file_path = plans_dir.join(format!("{name}.md"));

                if !file_path.exists() {
                    return Err(AgentError::ValidationError {
                        reason: format!("no plan named '{name}' found in .stencila/plans/"),
                    });
                }

                let content =
                    tokio::fs::read_to_string(&file_path)
                        .await
                        .map_err(|e| AgentError::Io {
                            message: format!(
                                "failed to read plan file {}: {e}",
                                file_path.display()
                            ),
                        })?;

                let path_str = file_path.display().to_string();
                Ok(ToolOutput::Text(format!("Plan: {path_str}\n\n{content}")))
            })
        },
    )
}

/// Find the name of the most recently written plan.
fn find_latest_name(plans_dir: &Path) -> Result<String, AgentError> {
    let entries = std::fs::read_dir(plans_dir).map_err(|e| AgentError::Io {
        message: format!(
            "failed to read plans directory {}: {e}",
            plans_dir.display()
        ),
    })?;

    let mut newest: Option<(PathBuf, SystemTime)> = None;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("md") {
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
        reason: "no plan files found in .stencila/plans/".into(),
    })?;

    let name =
        path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| AgentError::ValidationError {
                reason: "invalid plan filename".into(),
            })?;

    Ok(name.to_string())
}
