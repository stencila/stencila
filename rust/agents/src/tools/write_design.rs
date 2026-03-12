//! `write_design` tool: persist a software design specification.

use std::path::Path;

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::error::AgentError;
use crate::registry::{ToolExecutorFn, ToolOutput};

use super::required_str;

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

                let path_str = file_path.display().to_string();
                Ok(ToolOutput::Text(format!("Design written to {path_str}")))
            })
        },
    )
}
