//! `list_plans` tool: discover persisted implementation plans.

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
        name: "list_plans".into(),
        description: "List persisted implementation plans from .stencila/plans/. \
            Returns plan names in reverse chronological order (most recently written first)."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {},
            "required": [],
            "additionalProperties": false
        }),
        strict: false,
    }
}

pub fn executor() -> ToolExecutorFn {
    Box::new(
        |_args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move {
                let cwd = Path::new(env.working_directory());
                let stencila_dir = stencila_dirs::closest_stencila_dir(cwd, false)
                    .await
                    .map_err(|e| AgentError::Io {
                        message: format!("failed to find .stencila directory: {e}"),
                    })?;
                let plans_dir = stencila_dir.join("plans");

                if !plans_dir.exists() {
                    return Ok(ToolOutput::Text("No plans found.".into()));
                }

                let mut plans = list_plans_by_modified_time(&plans_dir)?;

                if plans.is_empty() {
                    return Ok(ToolOutput::Text("No plans found.".into()));
                }

                let entries: Vec<Value> = plans
                    .drain(..)
                    .enumerate()
                    .map(|(i, (name, _))| {
                        json!({
                            "name": name,
                            "path": plans_dir.join(format!("{name}.md")).display().to_string(),
                            "order": i + 1,
                        })
                    })
                    .collect();

                Ok(ToolOutput::Text(
                    serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "[]".to_string()),
                ))
            })
        },
    )
}

fn list_plans_by_modified_time(plans_dir: &Path) -> Result<Vec<(String, SystemTime)>, AgentError> {
    let entries = std::fs::read_dir(plans_dir).map_err(|e| AgentError::Io {
        message: format!(
            "failed to read plans directory {}: {e}",
            plans_dir.display()
        ),
    })?;

    let mut plans = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path: PathBuf = entry.path();

            if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                return None;
            }

            let modified = path.metadata().ok()?.modified().ok()?;
            let name = path.file_stem()?.to_str()?.to_string();

            Some((name, modified))
        })
        .collect::<Vec<_>>();

    plans.sort_by_key(|(_, modified)| std::cmp::Reverse(*modified));

    Ok(plans)
}
