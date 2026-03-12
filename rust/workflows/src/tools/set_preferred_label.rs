//! `set_preferred_label` tool: select a routing label for workflow branching.

use std::sync::{Arc, Mutex};

use serde_json::json;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};
use stencila_models3::types::tool::ToolDefinition;

fn definition(labels: &[String]) -> ToolDefinition {
    let labels_enum: Vec<serde_json::Value> = labels
        .iter()
        .map(|l| serde_json::Value::String(l.clone()))
        .collect();
    ToolDefinition {
        name: "set_preferred_label".into(),
        description: "Select which workflow branch to take next. Call this tool with one of the \
            available labels to route the workflow. You MUST call this tool exactly once before \
            finishing your response."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "label": {
                    "type": "string",
                    "enum": labels_enum,
                    "description": "The label of the outgoing edge to follow."
                }
            },
            "required": ["label"],
            "additionalProperties": false
        }),
        strict: true,
    }
}

fn executor(labels: Vec<String>, chosen: Arc<Mutex<Option<String>>>) -> ToolExecutorFn {
    Box::new(
        move |args: serde_json::Value,
              _env: &dyn stencila_agents::execution::ExecutionEnvironment| {
            let labels = labels.clone();
            let chosen = chosen.clone();
            Box::pin(async move {
                let label = args.get("label").and_then(|v| v.as_str()).ok_or_else(|| {
                    stencila_agents::error::AgentError::Io {
                        message: "Missing required parameter: label".to_string(),
                    }
                })?;

                let canonical = labels
                    .iter()
                    .find(|l| l.eq_ignore_ascii_case(label))
                    .cloned();

                let Some(canonical) = canonical else {
                    return Ok(ToolOutput::Text(format!(
                        "Error: '{label}' is not a valid label. Valid labels: {}",
                        labels.join(", ")
                    )));
                };

                // Poison recovery is safe: the guarded value is a simple Option<String>
                // with no invariants that could be violated by a panicking thread.
                let mut guard = chosen.lock().unwrap_or_else(|e| e.into_inner());
                *guard = Some(canonical.clone());

                Ok(ToolOutput::Text(format!("Routing set to: {canonical}")))
            })
        },
    )
}

pub fn registered_tool(labels: Vec<String>, chosen: Arc<Mutex<Option<String>>>) -> RegisteredTool {
    RegisteredTool::new(definition(&labels), executor(labels, chosen))
}
