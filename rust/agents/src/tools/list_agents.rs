//! `list_agents` tool: discover available Stencila Agents.

use std::path::Path;

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::registry::{ToolExecutorFn, ToolOutput};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_agents".into(),
        description: "List available agents from workspace, user config, and CLI-detected \
            sources. Returns agent names, descriptions, and sources to help decide which \
            agent to delegate to."
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
                let agents = crate::agent_def::discover(cwd).await;

                let entries: Vec<Value> = agents
                    .into_iter()
                    .map(|agent| {
                        json!({
                            "name": agent.name,
                            "description": agent.description,
                            "source": agent.source().map(|s| s.to_string()),
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
