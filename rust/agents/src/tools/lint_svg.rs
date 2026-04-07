//! `lint_svg` tool: statically analyze SVG overlays for layout,
//! reference, and attribute errors.

use serde_json::{Value, json};
use stencila_models3::types::tool::ToolDefinition;

use crate::error::AgentError;
use crate::registry::{ToolExecutorFn, ToolOutput};

use super::required_str;

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "lint_svg".into(),
        description:
            "Statically analyze an SVG overlay containing Stencila (s:*) custom elements for \
            layout collisions, out-of-bounds components, dangling anchor references, \
            unused anchors, invalid attributes, and other issues. Returns a JSON array \
            of diagnostic messages."
                .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "svg_content": {
                    "type": "string",
                    "description": "The SVG overlay source string containing s: custom elements to lint."
                }
            },
            "required": ["svg_content"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

pub fn executor() -> ToolExecutorFn {
    Box::new(|args: Value, _env| {
        Box::pin(async move {
            let content = required_str(&args, "svg_content")?;
            let result = stencila_svg_components::lint(content);

            let messages: Vec<Value> = result
                .messages
                .iter()
                .map(|m| {
                    json!({
                        "level": format!("{:?}", m.level),
                        "message": m.message,
                    })
                })
                .collect();

            let output = if messages.is_empty() {
                json!({ "status": "ok", "messages": [] })
            } else {
                json!({ "status": "issues_found", "count": messages.len(), "messages": messages })
            };

            let json_str = serde_json::to_string_pretty(&output).map_err(|e| AgentError::Io {
                message: e.to_string(),
            })?;

            Ok(ToolOutput::Text(json_str))
        })
    })
}
