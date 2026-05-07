//! `workflow_get_context` tool: read context key(s) or full snapshot.

use std::sync::{Arc, Mutex};

use serde_json::json;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};
use stencila_db::rusqlite::Connection;
use stencila_models3::types::tool::ToolDefinition;

fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "workflow_get_context".into(),
        description: "Read workflow context values. Provide `keys` to read one or more \
            context keys, or omit `keys` / provide an empty array to read all context. Stored \
            JSON values are parsed on read. Missing keys are omitted from the returned object."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "keys": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Context keys to read. Use a one-item array to read a single key. Omit or pass an empty array to return all context values."
                }
            },
            "additionalProperties": false
        }),
        strict: false,
    }
}

fn parse_context_value(value: String) -> serde_json::Value {
    serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value))
}

fn executor(conn: Arc<Mutex<Connection>>, run_id: String) -> ToolExecutorFn {
    Box::new(
        move |args: serde_json::Value,
              _env: &dyn stencila_agents::execution::ExecutionEnvironment| {
            let conn = conn.clone();
            let run_id = run_id.clone();
            Box::pin(async move {
                let conn = conn.lock().unwrap_or_else(|e| e.into_inner());

                let keys = args.get("keys").and_then(|v| v.as_array());

                if let Some(keys) = keys
                    && !keys.is_empty()
                {
                    // Key lookup
                    let mut stmt = conn
                        .prepare(
                            "SELECT value FROM workflow_context WHERE run_id = ?1 AND key = ?2",
                        )
                        .map_err(|e| stencila_agents::error::AgentError::Io {
                            message: format!("Failed to prepare query: {e}"),
                        })?;

                    let mut map = serde_json::Map::new();
                    for key in keys.iter().filter_map(|key| key.as_str()) {
                        let result = stmt.query_row((&run_id, key), |row| row.get::<_, String>(0));
                        match result {
                            Ok(value) => {
                                map.insert(key.to_string(), parse_context_value(value));
                            }
                            Err(stencila_db::rusqlite::Error::QueryReturnedNoRows) => {}
                            Err(e) => return Ok(ToolOutput::Text(format!("Error: {e}"))),
                        }
                    }

                    Ok(ToolOutput::Text(serde_json::Value::Object(map).to_string()))
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
                        map.insert(k, parse_context_value(v));
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

#[cfg(test)]
mod tests {
    use super::*;

    use stencila_agents::{execution::LocalExecutionEnvironment, registry::ToolOutput};

    fn conn_with_context() -> Arc<Mutex<Connection>> {
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
        conn.execute(
            "INSERT INTO workflow_context (run_id, key, value, updated_at) VALUES
                ('run-1', 'current_slice', '\"Slice 1\"', 'now'),
                ('run-1', 'slice.packages', '[\"rust/workflows\"]', 'now')",
            (),
        )
        .unwrap_or_else(|error| panic!("{error}"));
        Arc::new(Mutex::new(conn))
    }

    #[tokio::test]
    async fn gets_single_context_key_as_object() {
        let executor = executor(conn_with_context(), "run-1".to_string());
        let env = LocalExecutionEnvironment::new(".");

        let output = executor(json!({"keys": ["current_slice"]}), &env)
            .await
            .unwrap_or_else(|error| panic!("{error}"));

        let ToolOutput::Text(text) = output else {
            panic!("expected text output")
        };
        let value: serde_json::Value =
            serde_json::from_str(&text).unwrap_or_else(|error| panic!("{error}"));

        assert_eq!(value["current_slice"], "Slice 1");
    }

    #[tokio::test]
    async fn gets_multiple_context_keys() {
        let executor = executor(conn_with_context(), "run-1".to_string());
        let env = LocalExecutionEnvironment::new(".");

        let output = executor(
            json!({"keys": ["current_slice", "slice.packages", "missing"]}),
            &env,
        )
        .await
        .unwrap_or_else(|error| panic!("{error}"));

        let ToolOutput::Text(text) = output else {
            panic!("expected text output")
        };
        let value: serde_json::Value =
            serde_json::from_str(&text).unwrap_or_else(|error| panic!("{error}"));

        assert_eq!(value["current_slice"], "Slice 1");
        assert_eq!(value["slice.packages"], json!(["rust/workflows"]));
        assert!(value.get("missing").is_none());
    }

    #[tokio::test]
    async fn gets_full_context_snapshot_for_empty_keys() {
        let executor = executor(conn_with_context(), "run-1".to_string());
        let env = LocalExecutionEnvironment::new(".");

        let output = executor(json!({"keys": []}), &env)
            .await
            .unwrap_or_else(|error| panic!("{error}"));

        let ToolOutput::Text(text) = output else {
            panic!("expected text output")
        };
        let value: serde_json::Value =
            serde_json::from_str(&text).unwrap_or_else(|error| panic!("{error}"));

        assert_eq!(value["current_slice"], "Slice 1");
        assert_eq!(value["slice.packages"], json!(["rust/workflows"]));
    }

    #[tokio::test]
    async fn gets_full_context_snapshot() {
        let executor = executor(conn_with_context(), "run-1".to_string());
        let env = LocalExecutionEnvironment::new(".");

        let output = executor(json!({}), &env)
            .await
            .unwrap_or_else(|error| panic!("{error}"));

        let ToolOutput::Text(text) = output else {
            panic!("expected text output")
        };
        let value: serde_json::Value =
            serde_json::from_str(&text).unwrap_or_else(|error| panic!("{error}"));

        assert_eq!(value["current_slice"], "Slice 1");
        assert_eq!(value["slice.packages"], json!(["rust/workflows"]));
    }
}
