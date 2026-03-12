//! `write_plan` tool: persist an implementation plan.

use std::path::Path;

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::error::AgentError;
use crate::registry::{ToolExecutorFn, ToolOutput};

use super::{required_str, to_kebab_case_name};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "write_plan".into(),
        description: "Persist an implementation plan as a Markdown file in \
            .stencila/plans/. Accepts a plan name (coerced to kebab-case) and the \
            full Markdown content. Returns the path of the written file."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Plan name, coerced to kebab-case (e.g. 'user-auth-flow')."
                },
                "content": {
                    "type": "string",
                    "description": "Full Markdown content of the implementation plan."
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
                let name = to_kebab_case_name(required_str(&args, "name")?)?;
                let content = required_str(&args, "content")?;

                let cwd = Path::new(env.working_directory());
                let stencila_dir = stencila_dirs::closest_stencila_dir(cwd, true)
                    .await
                    .map_err(|e| AgentError::Io {
                        message: format!("failed to find .stencila directory: {e}"),
                    })?;
                let plans_dir = stencila_dir.join("plans");

                // Ensure the plans directory exists
                tokio::fs::create_dir_all(&plans_dir)
                    .await
                    .map_err(|e| AgentError::Io {
                        message: format!(
                            "failed to create plans directory {}: {e}",
                            plans_dir.display()
                        ),
                    })?;

                let filename = format!("{name}.md");
                let file_path = plans_dir.join(&filename);

                tokio::fs::write(&file_path, content)
                    .await
                    .map_err(|e| AgentError::Io {
                        message: format!("failed to write plan file {}: {e}", file_path.display()),
                    })?;

                let path_str = file_path.display().to_string();
                Ok(ToolOutput::Text(format!("Plan written to {path_str}")))
            })
        },
    )
}
