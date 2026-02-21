//! `list_completed_nodes` tool: list completed nodes with their statuses.

use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use serde_json::json;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};
use stencila_models3::types::tool::ToolDefinition;

fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_completed_nodes".into(),
        description: "List all completed pipeline nodes with their status and duration.".into(),
        parameters: json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }),
        strict: false,
    }
}

fn executor(conn: Arc<Mutex<Connection>>, run_id: String) -> ToolExecutorFn {
    Box::new(
        move |_args: serde_json::Value,
              _env: &dyn stencila_agents::execution::ExecutionEnvironment| {
            let conn = conn.clone();
            let run_id = run_id.clone();
            Box::pin(async move {
                let conn = conn.lock().unwrap_or_else(|e| e.into_inner());
                let mut stmt = conn
                    .prepare(
                        "SELECT node_id, status, duration_ms FROM workflow_nodes \
                         WHERE run_id = ?1 ORDER BY started_at",
                    )
                    .map_err(|e| stencila_agents::error::AgentError::Io {
                        message: format!("Failed to prepare query: {e}"),
                    })?;

                let nodes: Vec<serde_json::Value> = stmt
                    .query_map((&run_id,), |row| {
                        Ok(json!({
                            "node_id": row.get::<_, String>(0)?,
                            "status": row.get::<_, String>(1)?,
                            "duration_ms": row.get::<_, Option<i64>>(2)?,
                        }))
                    })
                    .map_err(|e| stencila_agents::error::AgentError::Io {
                        message: format!("Failed to query nodes: {e}"),
                    })?
                    .filter_map(Result::ok)
                    .collect();

                Ok(ToolOutput::Text(
                    serde_json::Value::Array(nodes).to_string(),
                ))
            })
        },
    )
}

pub fn registered_tool(conn: Arc<Mutex<Connection>>, run_id: String) -> RegisteredTool {
    RegisteredTool::new(definition(), executor(conn, run_id))
}
