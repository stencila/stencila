//! `store_artifact` tool: register a workflow artifact in SQLite.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::json;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};
use stencila_db::rusqlite::Connection;
use stencila_models3::types::tool::ToolDefinition;

fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "store_artifact".into(),
        description: "Store metadata for a workflow artifact and return its artifact ID.".into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Human-readable artifact name."
                },
                "path": {
                    "type": "string",
                    "description": "Path to an existing file artifact."
                },
                "mime_type": {
                    "type": "string",
                    "description": "Optional MIME type for the artifact."
                }
            },
            "required": ["name", "path"],
            "additionalProperties": false
        }),
        strict: false,
    }
}

fn executor(
    conn: Arc<Mutex<Connection>>,
    run_id: String,
    artifacts_dir: PathBuf,
    workspace_root: PathBuf,
) -> ToolExecutorFn {
    Box::new(
        move |args: serde_json::Value,
              _env: &dyn stencila_agents::execution::ExecutionEnvironment| {
            let conn = conn.clone();
            let run_id = run_id.clone();
            let artifacts_dir = artifacts_dir.clone();
            let workspace_root = workspace_root.clone();
            Box::pin(async move {
                let name = args.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
                    stencila_agents::error::AgentError::Io {
                        message: "Missing required parameter: name".to_string(),
                    }
                })?;
                let input_path = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
                    stencila_agents::error::AgentError::Io {
                        message: "Missing required parameter: path".to_string(),
                    }
                })?;
                let mime_type = args.get("mime_type").and_then(|v| v.as_str());

                let artifact_id = uuid::Uuid::now_v7().to_string();
                let source = resolve_path(input_path);
                if !source.exists() || !source.is_file() {
                    return Ok(ToolOutput::Text(format!(
                        "Error: Artifact path not found or not a file: {}",
                        source.display()
                    )));
                }

                if let Err(error) = std::fs::create_dir_all(&artifacts_dir) {
                    return Ok(ToolOutput::Text(format!(
                        "Error creating artifact directory: {error}"
                    )));
                }

                // Ensure artifacts for a run are stored under
                // `.stencila/artifacts/workflows/{run_id}/`.
                let file_name = source
                    .file_name()
                    .map_or_else(|| "artifact.bin".into(), |name| name.to_os_string());
                let dest =
                    artifacts_dir.join(format!("{}-{}", artifact_id, file_name.to_string_lossy()));
                if let Err(error) = std::fs::copy(&source, &dest) {
                    return Ok(ToolOutput::Text(format!(
                        "Error copying artifact file: {error}"
                    )));
                }

                let size_bytes = match std::fs::metadata(&dest) {
                    Ok(meta) => i64::try_from(meta.len()).ok(),
                    Err(_) => None,
                };

                let rel = match workspace_relative_path(&workspace_root, &dest) {
                    Some(path) => path,
                    None => {
                        return Ok(ToolOutput::Text(format!(
                            "Error: Artifact path is outside workspace: {}",
                            dest.display()
                        )));
                    }
                };

                let conn = conn.lock().unwrap_or_else(|e| e.into_inner());
                let result = conn.execute(
                    "INSERT INTO workflow_artifacts (run_id, artifact_id, name, mime_type, size_bytes, path)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    (
                        &run_id,
                        &artifact_id,
                        name,
                        mime_type,
                        size_bytes,
                        rel.to_string_lossy().to_string(),
                    ),
                );

                match result {
                    Ok(_) => {
                        let response = json!({
                            "artifact_id": artifact_id,
                            "name": name,
                            "path": rel,
                            "mime_type": mime_type,
                            "size_bytes": size_bytes
                        });
                        Ok(ToolOutput::Text(response.to_string()))
                    }
                    Err(e) => Ok(ToolOutput::Text(format!("Error storing artifact: {e}"))),
                }
            })
        },
    )
}

fn resolve_path(path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else if let Ok(cwd) = std::env::current_dir() {
        cwd.join(p)
    } else {
        p
    }
}

fn workspace_relative_path(workspace_root: &Path, path: &Path) -> Option<PathBuf> {
    let root = workspace_root.canonicalize().ok()?;
    let full = path.canonicalize().ok()?;
    full.strip_prefix(root).ok().map(ToOwned::to_owned)
}

pub fn registered_tool(
    conn: Arc<Mutex<Connection>>,
    run_id: String,
    artifacts_dir: PathBuf,
    workspace_root: PathBuf,
) -> RegisteredTool {
    RegisteredTool::new(
        definition(),
        executor(conn, run_id, artifacts_dir, workspace_root),
    )
}
