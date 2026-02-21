//! `set_workflow_context` tool: write a context key-value pair.

use std::sync::{Arc, Mutex};

use stencila_db::rusqlite::Connection;
use serde_json::json;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};
use stencila_models3::types::tool::ToolDefinition;

fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "set_workflow_context".into(),
        description: "Set a workflow context value. The value will be available to downstream \
            nodes and edge conditions. LLM-managed keys are stored under the `llm.` namespace. \
            Requires the current node to have context_writable=true."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "Context key to set."
                },
                "value": {
                    "description": "Value to store (string, number, boolean, or JSON)."
                }
            },
            "required": ["key", "value"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

fn executor(
    conn: Arc<Mutex<Connection>>,
    run_id: String,
    context_writable: bool,
) -> ToolExecutorFn {
    Box::new(
        move |args: serde_json::Value,
              _env: &dyn stencila_agents::execution::ExecutionEnvironment| {
            let conn = conn.clone();
            let run_id = run_id.clone();
            let writable = context_writable;
            Box::pin(async move {
                if !writable {
                    return Ok(ToolOutput::Text(
                        "Error: Context writes are not enabled for this node. \
                         The pipeline author must set context_writable=true on the node."
                            .to_string(),
                    ));
                }

                let key = args.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
                    stencila_agents::error::AgentError::Io {
                        message: "Missing required parameter: key".to_string(),
                    }
                })?;
                if key.starts_with("internal.") {
                    return Ok(ToolOutput::Text(
                        "Error: Keys starting with `internal.` are reserved.".to_string(),
                    ));
                }
                let storage_key = if key.starts_with("llm.") {
                    key.to_string()
                } else {
                    format!("llm.{key}")
                };

                let value = args
                    .get("value")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                let value_str = value.to_string();

                let conn = conn.lock().unwrap_or_else(|e| e.into_inner());
                match conn.execute(
                    "INSERT INTO workflow_context (run_id, key, value, updated_at)
                     VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                     ON CONFLICT(run_id, key) DO UPDATE SET
                        value = excluded.value,
                        updated_at = excluded.updated_at",
                    (&run_id, &storage_key, &value_str),
                ) {
                    Ok(_) => Ok(ToolOutput::Text(format!("Set context key: {storage_key}"))),
                    Err(e) => Ok(ToolOutput::Text(format!("Error setting context: {e}"))),
                }
            })
        },
    )
}

pub fn registered_tool(
    conn: Arc<Mutex<Connection>>,
    run_id: String,
    context_writable: bool,
) -> RegisteredTool {
    RegisteredTool::new(definition(), executor(conn, run_id, context_writable))
}
