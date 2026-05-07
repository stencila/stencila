//! `workflow_set_context` tool: write workflow context key-value pairs.

use std::sync::{Arc, Mutex};

use serde_json::json;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};
use stencila_db::rusqlite::Connection;
use stencila_models3::types::tool::ToolDefinition;

type AgentIoError = stencila_agents::error::AgentError;

fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "workflow_set_context".into(),
        description: "Set workflow context values. Values will be available to downstream \
            nodes and edge conditions. Keys starting with `internal.` are reserved. \
            Requires the current node to have context_writable=true. Provide one or more \
            key-value pairs using `entries`."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "entries": {
                    "type": "object",
                    "description": "Context key-value pairs to set. Use a one-property object to set a single key.",
                    "additionalProperties": true
                }
            },
            "required": ["entries"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

fn validate_context_key(key: &str) -> Option<ToolOutput> {
    if key.starts_with("internal.") {
        Some(ToolOutput::Text(format!(
            "Error: Key `{key}` starts with reserved prefix `internal.`."
        )))
    } else {
        None
    }
}

fn upsert_context(
    conn: &Connection,
    run_id: &str,
    key: &str,
    value: &serde_json::Value,
) -> stencila_db::rusqlite::Result<usize> {
    let value_str = value.to_string();
    conn.execute(
        "INSERT INTO workflow_context (run_id, key, value, updated_at)
         VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         ON CONFLICT(run_id, key) DO UPDATE SET
            value = excluded.value,
            updated_at = excluded.updated_at",
        (run_id, key, &value_str),
    )
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
                         The workflow author must set context_writable=true on the node."
                            .to_string(),
                    ));
                }

                let entries = args
                    .get("entries")
                    .and_then(|value| value.as_object())
                    .ok_or_else(|| AgentIoError::Io {
                        message: "Missing required parameter: entries".to_string(),
                    })?;

                for key in entries.keys() {
                    if let Some(error) = validate_context_key(key) {
                        return Ok(error);
                    }
                }

                let mut conn = conn.lock().unwrap_or_else(|e| e.into_inner());
                let tx = conn.transaction().map_err(|e| AgentIoError::Io {
                    message: format!("Failed to start context transaction: {e}"),
                })?;

                for (key, value) in entries {
                    if let Err(e) = upsert_context(&tx, &run_id, key, value) {
                        return Ok(ToolOutput::Text(format!("Error setting context: {e}")));
                    }
                }

                if let Err(e) = tx.commit() {
                    return Ok(ToolOutput::Text(format!("Error setting context: {e}")));
                }

                let mut keys = entries.keys().cloned().collect::<Vec<_>>();
                keys.sort();
                let message = if keys.is_empty() {
                    "Set context keys: none".to_string()
                } else {
                    format!("Set context keys: {}", keys.join(", "))
                };
                Ok(ToolOutput::Text(message))
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

#[cfg(test)]
mod tests {
    use super::*;

    use stencila_agents::{execution::LocalExecutionEnvironment, registry::ToolOutput};

    fn empty_context_conn() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap_or_else(|error| panic!("{error}"));
        conn.execute(
            "CREATE TABLE workflow_context (
                run_id TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (run_id, key)
            )",
            (),
        )
        .unwrap_or_else(|error| panic!("{error}"));
        Arc::new(Mutex::new(conn))
    }

    #[tokio::test]
    async fn sets_multiple_context_entries_atomically() {
        let conn = empty_context_conn();
        let executor = executor(conn.clone(), "run-1".to_string(), true);
        let env = LocalExecutionEnvironment::new(".");

        let output = executor(
            json!({"entries": {"current_slice": "Slice 1", "slice.packages": ["rust/workflows"]}}),
            &env,
        )
        .await
        .unwrap_or_else(|error| panic!("{error}"));

        let ToolOutput::Text(text) = output else {
            panic!("expected text output")
        };
        assert!(text.contains("current_slice"));
        assert!(text.contains("slice.packages"));

        let conn = conn.lock().unwrap_or_else(|error| error.into_inner());
        let current_slice: String = conn
            .query_row(
                "SELECT value FROM workflow_context WHERE run_id = 'run-1' AND key = 'current_slice'",
                (),
                |row| row.get(0),
            )
            .unwrap_or_else(|error| panic!("{error}"));
        let packages: String = conn
            .query_row(
                "SELECT value FROM workflow_context WHERE run_id = 'run-1' AND key = 'slice.packages'",
                (),
                |row| row.get(0),
            )
            .unwrap_or_else(|error| panic!("{error}"));

        assert_eq!(current_slice, "\"Slice 1\"");
        assert_eq!(packages, "[\"rust/workflows\"]");
    }

    #[tokio::test]
    async fn rejects_reserved_batch_keys_before_writing() {
        let conn = empty_context_conn();
        let executor = executor(conn.clone(), "run-1".to_string(), true);
        let env = LocalExecutionEnvironment::new(".");

        let output = executor(
            json!({"entries": {"visible": true, "internal.secret": "nope"}}),
            &env,
        )
        .await
        .unwrap_or_else(|error| panic!("{error}"));

        let ToolOutput::Text(text) = output else {
            panic!("expected text output")
        };
        assert_eq!(
            text,
            "Error: Key `internal.secret` starts with reserved prefix `internal.`."
        );

        let count: i64 = conn
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .query_row("SELECT COUNT(*) FROM workflow_context", (), |row| {
                row.get(0)
            })
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(count, 0);
    }
}
