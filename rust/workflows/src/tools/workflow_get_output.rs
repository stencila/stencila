//! `workflow_get_output` tool: read a node's LLM response, or the most recent output.

use std::sync::{Arc, Mutex};

use serde_json::json;
use stencila_agents::error::AgentError;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};
use stencila_attractor::types::StageStatus;
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
                let blob = {
                    let conn = conn.lock().unwrap_or_else(|e| e.into_inner());

                    if let Some(node_id) = args
                        .get("node_id")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                    {
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
                            Err(e) => {
                                return Err(AgentError::Io {
                                    message: format!(
                                        "Failed to get output for node `{node_id}`: {e}"
                                    ),
                                });
                            }
                        }
                    } else {
                        // Most recently completed node that produced output.
                        //
                        // Joins workflow_node_outputs (which only contains rows for
                        // nodes that actually produced output) with workflow_nodes
                        // (which tracks completion timestamps). This avoids reading a
                        // global context key that could be stale or overwritten.
                        //
                        // Node statuses use StageStatus::as_str() values. We match
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
                             WHERE o.run_id = ?1 AND n.status IN (?2, ?3, ?4) \
                             ORDER BY n.completed_at DESC, n.rowid DESC \
                             LIMIT 1",
                            (
                                &run_id,
                                StageStatus::Success.as_str(),
                                StageStatus::PartialSuccess.as_str(),
                                StageStatus::Fail.as_str(),
                            ),
                            |row| row.get::<_, Vec<u8>>(0),
                        );
                        match result {
                            Ok(blob) => blob,
                            Err(stencila_db::rusqlite::Error::QueryReturnedNoRows) => {
                                return Ok(ToolOutput::Text(
                                    "No previous output available.".to_string(),
                                ));
                            }
                            Err(e) => {
                                return Err(AgentError::Io {
                                    message: format!(
                                        "Failed to get most recent workflow output: {e}"
                                    ),
                                });
                            }
                        }
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

#[cfg(test)]
mod tests {
    use super::*;

    use stencila_agents::{execution::LocalExecutionEnvironment, registry::ToolOutput};

    fn output_conn() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap_or_else(|error| panic!("{error}"));
        conn.execute(
            "CREATE TABLE workflow_nodes (
                run_id         TEXT NOT NULL,
                node_id        TEXT NOT NULL,
                status         TEXT NOT NULL DEFAULT 'pending',
                started_at     TEXT,
                completed_at   TEXT,
                duration_ms    INTEGER,
                model          TEXT,
                provider       TEXT,
                input_tokens   INTEGER DEFAULT 0,
                output_tokens  INTEGER DEFAULT 0,
                retry_count    INTEGER DEFAULT 0,
                failure_reason TEXT,
                PRIMARY KEY (run_id, node_id)
            )",
            (),
        )
        .unwrap_or_else(|error| panic!("{error}"));
        conn.execute(
            "CREATE TABLE workflow_node_outputs (
                run_id   TEXT NOT NULL,
                node_id  TEXT NOT NULL,
                output   BLOB NOT NULL,
                PRIMARY KEY (run_id, node_id)
            )",
            (),
        )
        .unwrap_or_else(|error| panic!("{error}"));
        Arc::new(Mutex::new(conn))
    }

    fn compressed(text: &str) -> Vec<u8> {
        zstd::encode_all(std::io::Cursor::new(text.as_bytes()), 0)
            .unwrap_or_else(|error| panic!("{error}"))
    }

    #[tokio::test]
    async fn gets_most_recent_terminal_output() {
        let conn = output_conn();
        {
            let conn = conn.lock().unwrap_or_else(|error| error.into_inner());
            conn.execute(
                "INSERT INTO workflow_nodes (run_id, node_id, status, completed_at) VALUES
                    ('run-1', 'first', ?1, '2026-01-01T00:00:00.000Z'),
                    ('run-1', 'latest', ?2, '2026-01-01T00:01:00.000Z'),
                    ('run-1', 'retrying', ?3, '2026-01-01T00:02:00.000Z')",
                (
                    StageStatus::Success.as_str(),
                    StageStatus::PartialSuccess.as_str(),
                    StageStatus::Retry.as_str(),
                ),
            )
            .unwrap_or_else(|error| panic!("{error}"));
            conn.execute(
                "INSERT INTO workflow_node_outputs (run_id, node_id, output) VALUES
                    ('run-1', 'first', ?1),
                    ('run-1', 'latest', ?2),
                    ('run-1', 'retrying', ?3)",
                (
                    compressed("first"),
                    compressed("latest"),
                    compressed("retrying"),
                ),
            )
            .unwrap_or_else(|error| panic!("{error}"));
        }

        let executor = executor(conn, "run-1".to_string());
        let env = LocalExecutionEnvironment::new(".");

        let output = executor(json!({}), &env)
            .await
            .unwrap_or_else(|error| panic!("{error}"));

        let ToolOutput::Text(text) = output else {
            panic!("expected text output")
        };
        assert_eq!(text, "latest");
    }
}
