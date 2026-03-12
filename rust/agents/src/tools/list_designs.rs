//! `list_designs` tool: discover persisted software design specifications.

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
        name: "list_designs".into(),
        description: "List persisted software design specifications from .stencila/designs/. \
            Returns design names in reverse chronological order (most recently written first)."
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
                let designs_dir = stencila_dir.join("designs");

                if !designs_dir.exists() {
                    return Ok(ToolOutput::Text("No designs found.".into()));
                }

                let mut designs = list_designs_by_modified_time(&designs_dir)?;

                if designs.is_empty() {
                    return Ok(ToolOutput::Text("No designs found.".into()));
                }

                let entries: Vec<Value> = designs
                    .drain(..)
                    .enumerate()
                    .map(|(i, (name, _))| {
                        json!({
                            "name": name,
                            "path": designs_dir.join(format!("{name}.md")).display().to_string(),
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

fn list_designs_by_modified_time(
    designs_dir: &Path,
) -> Result<Vec<(String, SystemTime)>, AgentError> {
    let entries = std::fs::read_dir(designs_dir).map_err(|e| AgentError::Io {
        message: format!(
            "failed to read designs directory {}: {e}",
            designs_dir.display()
        ),
    })?;

    let mut designs = entries
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

    designs.sort_by_key(|(_, modified)| std::cmp::Reverse(*modified));

    Ok(designs)
}
