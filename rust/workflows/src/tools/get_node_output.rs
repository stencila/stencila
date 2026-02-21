//! `get_node_output` tool: read a specific node's LLM response.

use std::sync::{Arc, Mutex};

use stencila_db::rusqlite::Connection;
use serde_json::json;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};
use stencila_models3::types::tool::ToolDefinition;

fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "get_node_output".into(),
        description: "Get the LLM response text from a specific pipeline node.".into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "node_id": {
                    "type": "string",
                    "description": "ID of the node whose output to retrieve."
                }
            },
            "required": ["node_id"],
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
                let node_id = args
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| stencila_agents::error::AgentError::Io {
                        message: "Missing required parameter: node_id".to_string(),
                    })?;

                let conn = conn.lock().unwrap_or_else(|e| e.into_inner());
                let result = conn.query_row(
                    "SELECT output FROM workflow_node_outputs WHERE run_id = ?1 AND node_id = ?2",
                    (&run_id, node_id),
                    |row| row.get::<_, Vec<u8>>(0),
                );

                match result {
                    Ok(blob) => {
                        match zstd::decode_all(std::io::Cursor::new(&blob)) {
                            Ok(decoded) => match String::from_utf8(decoded) {
                                Ok(text) => Ok(ToolOutput::Text(text)),
                                Err(_) => {
                                    Ok(ToolOutput::Text("<binary response data>".to_string()))
                                }
                            },
                            Err(_) => {
                                // Backward compatibility for previously uncompressed rows.
                                let text = String::from_utf8(blob)
                                    .unwrap_or_else(|_| "<binary response data>".to_string());
                                Ok(ToolOutput::Text(text))
                            }
                        }
                    }
                    Err(stencila_db::rusqlite::Error::QueryReturnedNoRows) => Ok(ToolOutput::Text(format!(
                        "No output found for node: {node_id}"
                    ))),
                    Err(e) => Ok(ToolOutput::Text(format!("Error: {e}"))),
                }
            })
        },
    )
}

pub fn registered_tool(conn: Arc<Mutex<Connection>>, run_id: String) -> RegisteredTool {
    RegisteredTool::new(definition(), executor(conn, run_id))
}
