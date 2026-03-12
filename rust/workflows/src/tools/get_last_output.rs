//! `get_last_output` tool: fetch the previous pipeline node's full output.

use std::sync::{Arc, Mutex};

use serde_json::json;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};
use stencila_db::rusqlite::Connection;
use stencila_models3::types::tool::ToolDefinition;

fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "get_last_output".into(),
        description: "Get the full output text from the most recently completed pipeline node. \
            Use this to retrieve reviewer feedback, prior drafts, or any other output from the \
            previous stage."
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

                // Find the most recently completed node that has stored output,
                // using the node-level `completed_at` timestamp from workflow_nodes
                // joined with workflow_node_outputs. This avoids reading a global
                // context key that could be stale or overwritten.
                let result = conn.query_row(
                    "SELECT o.output FROM workflow_node_outputs o \
                     JOIN workflow_nodes n ON o.run_id = n.run_id AND o.node_id = n.node_id \
                     WHERE o.run_id = ?1 AND n.status = 'completed' \
                     ORDER BY n.completed_at DESC \
                     LIMIT 1",
                    (&run_id,),
                    |row| row.get::<_, Vec<u8>>(0),
                );

                match result {
                    Ok(blob) => match zstd::decode_all(std::io::Cursor::new(&blob)) {
                        Ok(decoded) => match String::from_utf8(decoded) {
                            Ok(text) => Ok(ToolOutput::Text(text)),
                            Err(_) => Ok(ToolOutput::Text("<binary response data>".to_string())),
                        },
                        Err(_) => {
                            let text = String::from_utf8(blob)
                                .unwrap_or_else(|_| "<binary response data>".to_string());
                            Ok(ToolOutput::Text(text))
                        }
                    },
                    Err(stencila_db::rusqlite::Error::QueryReturnedNoRows) => Ok(ToolOutput::Text(
                        "No previous output available.".to_string(),
                    )),
                    Err(e) => Ok(ToolOutput::Text(format!("Error: {e}"))),
                }
            })
        },
    )
}

pub fn registered_tool(conn: Arc<Mutex<Connection>>, run_id: String) -> RegisteredTool {
    RegisteredTool::new(definition(), executor(conn, run_id))
}
