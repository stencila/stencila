//! `list_skills` tool: discover available Stencila Skills for routing.

use std::path::Path;

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::registry::{ToolExecutorFn, ToolOutput};

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_skills".into(),
        description: "List available Stencila Skills from workspace, provider-specific, and \
            builtin sources. Returns compact skill metadata to help decide whether to delegate \
            to a general agent with instructions to use a matching skill."
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
                let skills =
                    stencila_skills::discover(cwd, &stencila_skills::SkillSource::all()).await;

                let entries: Vec<Value> = skills
                    .into_iter()
                    .map(|skill| {
                        let mut entry = json!({
                            "name": skill.name,
                            "description": skill.description,
                            "source": skill.source().to_string(),
                            "path": skill.path().display().to_string(),
                        });

                        if let Some(ref keywords) = skill.inner.options.keywords {
                            entry["keywords"] = json!(keywords);
                        }

                        if let Some(ref compatibility) = skill.compatibility {
                            entry["compatibility"] = json!(compatibility);
                        }

                        if let Some(ref allowed_tools) = skill.allowed_tools {
                            entry["allowedTools"] = json!(allowed_tools);
                        }

                        entry
                    })
                    .collect();

                Ok(ToolOutput::Text(
                    serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "[]".to_string()),
                ))
            })
        },
    )
}
