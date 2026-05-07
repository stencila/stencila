//! `workflow_store_artifact` tool: register a workflow artifact in SQLite.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::json;
use stencila_agents::error::AgentError;
use stencila_agents::registry::{RegisteredTool, ToolExecutorFn, ToolOutput};
use stencila_db::rusqlite::Connection;
use stencila_models3::types::tool::ToolDefinition;

fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "workflow_store_artifact".into(),
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
                    AgentError::ValidationError {
                        reason: "missing required parameter: name".to_string(),
                    }
                })?;
                let input_path = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
                    AgentError::ValidationError {
                        reason: "missing required parameter: path".to_string(),
                    }
                })?;
                let mime_type = args.get("mime_type").and_then(|v| v.as_str());

                let artifact_id = uuid::Uuid::now_v7().to_string();
                let source = resolve_path(input_path);
                if !source.exists() || !source.is_file() {
                    return Err(AgentError::FileNotFound {
                        path: source.display().to_string(),
                    });
                }

                if workspace_relative_path(&workspace_root, &source).is_none() {
                    return Err(AgentError::PermissionDenied {
                        path: source.display().to_string(),
                    });
                }

                std::fs::create_dir_all(&artifacts_dir).map_err(|error| AgentError::Io {
                    message: format!("Failed to create artifact directory: {error}"),
                })?;

                let artifact_dir_rel =
                    match workspace_relative_path(&workspace_root, &artifacts_dir) {
                        Some(path) => path,
                        None => {
                            return Err(AgentError::PermissionDenied {
                                path: artifacts_dir.display().to_string(),
                            });
                        }
                    };

                // Ensure artifacts for a run are stored under
                // `.stencila/artifacts/workflows/{run_id}/`.
                let file_name = source
                    .file_name()
                    .map_or_else(|| "artifact.bin".into(), |name| name.to_os_string());
                let dest =
                    artifacts_dir.join(format!("{}-{}", artifact_id, file_name.to_string_lossy()));
                std::fs::copy(&source, &dest).map_err(|error| AgentError::Io {
                    message: format!("Failed to copy artifact file: {error}"),
                })?;

                let size_bytes = match std::fs::metadata(&dest) {
                    Ok(meta) => i64::try_from(meta.len()).ok(),
                    Err(_) => None,
                };

                let rel = artifact_dir_rel.join(dest.file_name().unwrap_or(&file_name));

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
                    Err(e) => {
                        let cleanup_error = std::fs::remove_file(&dest).err();
                        let cleanup = cleanup_error.map_or_else(String::new, |error| {
                            format!("; also failed to remove copied artifact: {error}")
                        });
                        Err(AgentError::Io {
                            message: format!("Failed to store artifact metadata: {e}{cleanup}"),
                        })
                    }
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

#[cfg(test)]
mod tests {
    use super::*;

    use stencila_agents::{execution::LocalExecutionEnvironment, registry::ToolOutput};
    use tempfile::TempDir;

    fn artifacts_conn() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap_or_else(|error| panic!("{error}"));
        conn.execute(
            "CREATE TABLE workflow_artifacts (
                run_id      TEXT NOT NULL,
                artifact_id TEXT NOT NULL,
                name        TEXT NOT NULL,
                mime_type   TEXT,
                size_bytes  INTEGER,
                path        TEXT NOT NULL,
                stored_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                PRIMARY KEY (run_id, artifact_id)
            )",
            (),
        )
        .unwrap_or_else(|error| panic!("{error}"));
        Arc::new(Mutex::new(conn))
    }

    #[tokio::test]
    async fn rejects_artifact_source_outside_workspace() {
        let workspace = TempDir::new().unwrap_or_else(|error| panic!("{error}"));
        let outside = TempDir::new().unwrap_or_else(|error| panic!("{error}"));
        let outside_file = outside.path().join("secret.txt");
        std::fs::write(&outside_file, "secret").unwrap_or_else(|error| panic!("{error}"));

        let executor = executor(
            artifacts_conn(),
            "run-1".to_string(),
            workspace.path().join(".stencila/artifacts/workflows/run-1"),
            workspace.path().to_path_buf(),
        );
        let env = LocalExecutionEnvironment::new(workspace.path());

        let error = executor(
            json!({"name": "secret", "path": outside_file.to_string_lossy()}),
            &env,
        )
        .await
        .expect_err("outside-workspace artifact source should fail");

        assert!(matches!(error, AgentError::PermissionDenied { .. }));
    }

    #[tokio::test]
    async fn removes_copied_artifact_when_metadata_insert_fails() {
        let workspace = TempDir::new().unwrap_or_else(|error| panic!("{error}"));
        let source = workspace.path().join("artifact.txt");
        std::fs::write(&source, "artifact").unwrap_or_else(|error| panic!("{error}"));
        let artifacts_dir = workspace.path().join(".stencila/artifacts/workflows/run-1");

        let conn = Connection::open_in_memory().unwrap_or_else(|error| panic!("{error}"));
        let conn = Arc::new(Mutex::new(conn));
        let executor = executor(
            conn,
            "run-1".to_string(),
            artifacts_dir.clone(),
            workspace.path().to_path_buf(),
        );
        let env = LocalExecutionEnvironment::new(workspace.path());

        let error = executor(
            json!({"name": "artifact", "path": source.to_string_lossy()}),
            &env,
        )
        .await
        .expect_err("metadata insert should fail without workflow_artifacts table");

        assert!(matches!(error, AgentError::Io { .. }));
        let copied_files = std::fs::read_dir(&artifacts_dir)
            .unwrap_or_else(|error| panic!("{error}"))
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_else(|error| panic!("{error}"));
        assert!(copied_files.is_empty());
    }

    #[tokio::test]
    async fn stores_artifact_from_workspace_source() {
        let workspace = TempDir::new().unwrap_or_else(|error| panic!("{error}"));
        let source = workspace.path().join("artifact.txt");
        std::fs::write(&source, "artifact").unwrap_or_else(|error| panic!("{error}"));
        let artifacts_dir = workspace.path().join(".stencila/artifacts/workflows/run-1");
        let conn = artifacts_conn();

        let executor = executor(
            conn,
            "run-1".to_string(),
            artifacts_dir,
            workspace.path().to_path_buf(),
        );
        let env = LocalExecutionEnvironment::new(workspace.path());

        let output = executor(
            json!({"name": "artifact", "path": source.to_string_lossy()}),
            &env,
        )
        .await
        .unwrap_or_else(|error| panic!("{error}"));

        let ToolOutput::Text(text) = output else {
            panic!("expected text output")
        };
        let value: serde_json::Value =
            serde_json::from_str(&text).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(value["name"], "artifact");
        assert!(
            value["path"]
                .as_str()
                .is_some_and(|path| { path.starts_with(".stencila/artifacts/workflows/run-1/") })
        );
    }
}
