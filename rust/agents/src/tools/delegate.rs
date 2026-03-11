//! `delegate` tool: delegate the current task to another agent or workflow.

use std::path::Path;

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::error::AgentError;
use crate::registry::{ToolExecutorFn, ToolOutput};

use super::required_str;

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "delegate".into(),
        description: "Delegate the current task to another agent or workflow. This ends \
            the current exchange and launches the delegatee. Use this after inspecting \
            available agents and workflows with list_agents and list_workflows."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "kind": {
                    "type": "string",
                    "enum": ["agent", "workflow"],
                    "description": "Whether to delegate to an agent or workflow."
                },
                "name": {
                    "type": "string",
                    "description": "Name of the agent or workflow to delegate to."
                },
                "reason": {
                    "type": "string",
                    "description": "Brief explanation of why this delegatee was chosen."
                },
                "instruction": {
                    "type": "string",
                    "description": "What the delegatee should accomplish. For agent delegation, phrase as a task. For workflow delegation, phrase as a goal."
                }
            },
            "required": ["kind", "name", "reason", "instruction"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

pub fn executor() -> ToolExecutorFn {
    Box::new(
        |args: Value, env: &dyn crate::execution::ExecutionEnvironment| {
            Box::pin(async move {
                let kind = required_str(&args, "kind")?;
                let name = required_str(&args, "name")?;
                let reason = required_str(&args, "reason")?;
                let instruction = required_str(&args, "instruction")?;

                if kind != "agent" && kind != "workflow" {
                    return Err(AgentError::ValidationError {
                        reason: format!("invalid kind '{kind}'; expected 'agent' or 'workflow'"),
                    });
                }

                let cwd = Path::new(env.working_directory());

                match kind {
                    "agent" => {
                        let agents = crate::definition::discover(cwd).await;
                        if !agents.iter().any(|a| a.name == name) {
                            return Err(AgentError::ValidationError {
                                reason: format!("agent '{name}' not found"),
                            });
                        }
                    }
                    "workflow" => {
                        let exists = if let Some(dot_dir) =
                            stencila_dirs::closest_dot_dir(cwd, ".stencila")
                        {
                            dot_dir
                                .join("workflows")
                                .join(name)
                                .join("WORKFLOW.md")
                                .exists()
                        } else {
                            false
                        };
                        if !exists {
                            return Err(AgentError::ValidationError {
                                reason: format!("workflow '{name}' not found"),
                            });
                        }
                    }
                    _ => unreachable!(),
                }

                let result = json!({
                    "delegated": true,
                    "kind": kind,
                    "name": name,
                    "reason": reason,
                    "instruction": instruction,
                });

                Ok(ToolOutput::Text(
                    serde_json::to_string_pretty(&result)
                        .unwrap_or_else(|_| r#"{"delegated":true}"#.to_string()),
                ))
            })
        },
    )
}
