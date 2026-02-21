//! `get_artifact` tool: retrieve artifact metadata and path.

use std::sync::{Arc, Mutex};

use serde_json::json;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};
use stencila_db::rusqlite::Connection;
use stencila_models3::types::tool::ToolDefinition;

fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "get_artifact".into(),
        description: "Get metadata and file path for a named artifact from this pipeline run."
            .into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "artifact_id": {
                    "type": "string",
                    "description": "ID of the artifact to retrieve."
                }
            },
            "required": ["artifact_id"],
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
                let artifact_id = args
                    .get("artifact_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| stencila_agents::error::AgentError::Io {
                        message: "Missing required parameter: artifact_id".to_string(),
                    })?;

                let conn = conn.lock().unwrap_or_else(|e| e.into_inner());
                let result = conn.query_row(
                    "SELECT name, mime_type, size_bytes, path FROM workflow_artifacts \
                     WHERE run_id = ?1 AND artifact_id = ?2",
                    (&run_id, artifact_id),
                    |row| {
                        Ok(json!({
                            "artifact_id": artifact_id,
                            "name": row.get::<_, String>(0)?,
                            "mime_type": row.get::<_, Option<String>>(1)?,
                            "size_bytes": row.get::<_, Option<i64>>(2)?,
                            "path": row.get::<_, String>(3)?,
                        }))
                    },
                );

                match result {
                    Ok(info) => Ok(ToolOutput::Text(info.to_string())),
                    Err(stencila_db::rusqlite::Error::QueryReturnedNoRows) => Ok(ToolOutput::Text(
                        format!("Artifact not found: {artifact_id}"),
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
