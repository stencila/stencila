//! `list_designs` tool: discover persisted software design specifications.

use std::path::Path;

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::error::AgentError;
use crate::registry::{ToolExecutorFn, ToolOutput};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_designs".into(),
        description: "List persisted software design specifications from .stencila/designs/. \
            Returns design names in chronological order (most recently written last)."
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

                let index = super::write_design::read_index(&designs_dir).await;

                if index.is_empty() {
                    return Ok(ToolOutput::Text("No designs found.".into()));
                }

                let entries: Vec<Value> = index
                    .iter()
                    .enumerate()
                    .map(|(i, name)| {
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
