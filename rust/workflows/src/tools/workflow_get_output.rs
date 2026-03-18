//! `workflow_get_output` tool: read a node's LLM response, or the most recent output.

use std::sync::{Arc, Mutex};

use serde_json::json;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};
use stencila_db::rusqlite::Connection;
use stencila_models3::types::tool::ToolDefinition;

fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "workflow_get_output".into(),
        description: "Get the LLM response text from a workflow node. If node_id is provided, \
            returns that node's output. If omitted, returns the output from the most recently \
            completed node that produced output (useful for retrieving reviewer feedback, prior \
            drafts, or other output from a previous workflow stage)."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "node_id": {
                    "type": "string",
                    "description": "ID of the node whose output to retrieve. If omitted, returns the most recent output."
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

                let blob = if let Some(node_id) = args.get("node_id").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                    // Specific node lookup
                    let result = conn.query_row(
                        "SELECT output FROM workflow_node_outputs WHERE run_id = ?1 AND node_id = ?2",
                        (&run_id, node_id),
                        |row| row.get::<_, Vec<u8>>(0),
                    );
                    match result {
                        Ok(blob) => blob,
                        Err(stencila_db::rusqlite::Error::QueryReturnedNoRows) => {
                            return Ok(ToolOutput::Text(format!(
                                "No output found for node: {node_id}"
                            )));
                        }
                        Err(e) => return Ok(ToolOutput::Text(format!("Error: {e}"))),
                    }
                } else {
                    // Most recently completed node that produced output.
                    //
                    // Joins workflow_node_outputs (which only contains rows for
                    // nodes that actually produced output) with workflow_nodes
                    // (which tracks completion timestamps). This avoids reading a
                    // global context key that could be stale or overwritten.
                    //
                    // Node statuses use StageStatus::as_str() values: "success",
                    // "partial_success", "fail", "skipped", "retry". We match
                    // terminal statuses that indicate the node finished and could
                    // have produced output. "skipped" nodes are excluded because
                    // they don't run and have no meaningful output (and the JOIN
                    // on workflow_node_outputs would filter them anyway).
                    //
                    // The secondary sort on `n.rowid DESC` breaks ties when two
                    // nodes share the same `completed_at` timestamp (SQLite's
                    // `strftime` has limited sub-second precision).
                    let result = conn.query_row(
                        "SELECT o.output FROM workflow_node_outputs o \
                         JOIN workflow_nodes n ON o.run_id = n.run_id AND o.node_id = n.node_id \
                         WHERE o.run_id = ?1 AND n.status IN ('success', 'partial_success', 'fail') \
                         ORDER BY n.completed_at DESC, n.rowid DESC \
                         LIMIT 1",
                        (&run_id,),
                        |row| row.get::<_, Vec<u8>>(0),
                    );
                    match result {
                        Ok(blob) => blob,
                        Err(stencila_db::rusqlite::Error::QueryReturnedNoRows) => {
                            return Ok(ToolOutput::Text(
                                "No previous output available.".to_string(),
                            ));
                        }
                        Err(e) => return Ok(ToolOutput::Text(format!("Error: {e}"))),
                    }
                };

                // Decompress and decode the output blob.
                match zstd::decode_all(std::io::Cursor::new(&blob)) {
                    Ok(decoded) => match String::from_utf8(decoded) {
                        Ok(text) => Ok(ToolOutput::Text(text)),
                        Err(_) => Ok(ToolOutput::Text("<binary response data>".to_string())),
                    },
                    Err(_) => {
                        // Backward compatibility for previously uncompressed rows.
                        let text = String::from_utf8(blob)
                            .unwrap_or_else(|_| "<binary response data>".to_string());
                        Ok(ToolOutput::Text(text))
                    }
                }
            })
        },
    )
}

pub fn registered_tool(conn: Arc<Mutex<Connection>>, run_id: String) -> RegisteredTool {
    RegisteredTool::new(definition(), executor(conn, run_id))
}
