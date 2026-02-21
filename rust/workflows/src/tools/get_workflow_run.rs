//! `get_workflow_run` tool: read workflow run metadata.

use std::sync::{Arc, Mutex};

use stencila_db::rusqlite::Connection;
use serde_json::json;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};
use stencila_models3::types::tool::ToolDefinition;

fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "get_workflow_run".into(),
        description:
            "Get metadata about the current workflow run: name, goal, status, and start time."
                .into(),
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
                let result = conn.query_row(
                    "SELECT workflow_name, goal, status, started_at FROM workflow_runs WHERE run_id = ?1",
                    (&run_id,),
                    |row| {
                        Ok(json!({
                            "workflow_name": row.get::<_, String>(0)?,
                            "goal": row.get::<_, String>(1)?,
                            "status": row.get::<_, String>(2)?,
                            "started_at": row.get::<_, String>(3)?,
                            "run_id": run_id,
                        }))
                    },
                );
                match result {
                    Ok(info) => Ok(ToolOutput::Text(info.to_string())),
                    Err(e) => Ok(ToolOutput::Text(format!("Error: {e}"))),
                }
            })
        },
    )
}

pub fn registered_tool(conn: Arc<Mutex<Connection>>, run_id: String) -> RegisteredTool {
    RegisteredTool::new(definition(), executor(conn, run_id))
}
