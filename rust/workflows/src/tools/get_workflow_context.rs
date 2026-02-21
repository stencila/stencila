//! `get_workflow_context` tool: read context key(s) or full snapshot.

use std::sync::{Arc, Mutex};

use serde_json::json;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};
use stencila_db::rusqlite::Connection;
use stencila_models3::types::tool::ToolDefinition;

fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "get_workflow_context".into(),
        description: "Read a workflow context value by key, or get all context values if no \
            key is specified."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "Context key to read. If omitted, returns all context values."
                }
            },
            "additionalProperties": false
        }),
        strict: false,
    }
}

fn executor(conn: Arc<Mutex<Connection>>, run_id: String) -> ToolExecutorFn {
    Box::new(
        move |args: serde_json::Value,
              _env: &dyn stencila_agents::execution::ExecutionEnvironment| {
            let conn = conn.clone();
            let run_id = run_id.clone();
            Box::pin(async move {
                let conn = conn.lock().unwrap_or_else(|e| e.into_inner());

                if let Some(key) = args.get("key").and_then(|v| v.as_str()) {
                    // Single key lookup
                    let result = conn.query_row(
                        "SELECT value FROM workflow_context WHERE run_id = ?1 AND key = ?2",
                        (&run_id, key),
                        |row| row.get::<_, String>(0),
                    );
                    match result {
                        Ok(value) => Ok(ToolOutput::Text(value)),
                        Err(stencila_db::rusqlite::Error::QueryReturnedNoRows) => {
                            Ok(ToolOutput::Text(format!("Key not found: {key}")))
                        }
                        Err(e) => Ok(ToolOutput::Text(format!("Error: {e}"))),
                    }
                } else {
                    // Full snapshot
                    let mut stmt = conn
                        .prepare("SELECT key, value FROM workflow_context WHERE run_id = ?1 ORDER BY rowid")
                        .map_err(|e| stencila_agents::error::AgentError::Io {
                            message: format!("Failed to prepare query: {e}"),
                        })?;

                    let rows: Vec<(String, String)> = stmt
                        .query_map((&run_id,), |row| {
                            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                        })
                        .map_err(|e| stencila_agents::error::AgentError::Io {
                            message: format!("Failed to query context: {e}"),
                        })?
                        .filter_map(Result::ok)
                        .collect();

                    let mut map = serde_json::Map::new();
                    for (k, v) in rows {
                        let parsed =
                            serde_json::from_str(&v).unwrap_or(serde_json::Value::String(v));
                        map.insert(k, parsed);
                    }
                    Ok(ToolOutput::Text(serde_json::Value::Object(map).to_string()))
                }
            })
        },
    )
}

pub fn registered_tool(conn: Arc<Mutex<Connection>>, run_id: String) -> RegisteredTool {
    RegisteredTool::new(definition(), executor(conn, run_id))
}
