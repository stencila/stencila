//! SQLite-backed context storage.
//!
//! Implements [`ContextBackend`] using a `SQLite` database, enabling persistent
//! pipeline state across runs. This is a Stencila extension to the attractor
//! spec (§5). Feature-gated behind `sqlite`.

use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use indexmap::IndexMap;
use serde_json::Value;
use stencila_db::WorkspaceDb;
use stencila_db::migration::Migration;
use stencila_db::rusqlite::Connection;

use crate::context::ContextBackend;

// ---------------------------------------------------------------------------
// Workflow migrations
// ---------------------------------------------------------------------------

/// Migrations for the `workflows` domain.
///
/// These are registered with [`WorkspaceDb::migrate`] when a
/// [`SqliteBackend`] is opened.
pub static WORKFLOW_MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "initial",
    sql: include_str!("migrations/001_initial.sql"),
}];

// ---------------------------------------------------------------------------
// NodeRecord
// ---------------------------------------------------------------------------

/// Per-node execution data for [`SqliteBackend::upsert_node`].
pub struct NodeRecord<'a> {
    pub node_id: &'a str,
    pub status: &'a str,
    pub model: Option<&'a str>,
    pub provider: Option<&'a str>,
    pub duration_ms: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub retry_count: Option<i64>,
    pub failure_reason: Option<&'a str>,
}

// ---------------------------------------------------------------------------
// SqliteBackend
// ---------------------------------------------------------------------------

/// A [`ContextBackend`] backed by `SQLite`.
///
/// Reads and writes the `context` table scoped by `run_id`. The same
/// connection is shared (via `Arc<Mutex<Connection>>`) with tool executors
/// that need to query other tables (nodes, edges, etc.).
pub struct SqliteBackend {
    conn: Arc<Mutex<Connection>>,
    run_id: String,
}

impl fmt::Debug for SqliteBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteBackend")
            .field("run_id", &self.run_id)
            .finish_non_exhaustive()
    }
}

impl SqliteBackend {
    /// Open a backend scoped to `run_id` using an existing [`WorkspaceDb`].
    ///
    /// Runs the `workflows` domain migrations on the shared database and
    /// returns a backend that operates on that connection.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if migrations fail.
    pub fn open(
        workspace_db: &WorkspaceDb,
        run_id: &str,
    ) -> Result<Self, stencila_db::rusqlite::Error> {
        workspace_db.migrate("workflows", WORKFLOW_MIGRATIONS)?;

        Ok(Self {
            conn: workspace_db.connection().clone(),
            run_id: run_id.to_string(),
        })
    }

    /// Create a backend from an existing shared connection.
    ///
    /// Useful when the connection is already open and migrations have
    /// been applied.
    pub fn from_shared(conn: Arc<Mutex<Connection>>, run_id: String) -> Self {
        Self { conn, run_id }
    }

    /// Get the shared database connection handle.
    ///
    /// Tool executors capture this `Arc` to query the DB alongside the
    /// context backend.
    #[must_use]
    pub fn connection(&self) -> &Arc<Mutex<Connection>> {
        &self.conn
    }

    /// The run ID this backend is scoped to.
    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    // -- Run lifecycle methods ------------------------------------------------

    /// Insert a new run record.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` on database failure.
    pub fn insert_run(
        &self,
        workflow_name: &str,
        goal: &str,
        stencila_version: &str,
    ) -> Result<(), stencila_db::rusqlite::Error> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "INSERT INTO workflow_runs (run_id, workflow_name, goal, stencila_version)
             VALUES (?1, ?2, ?3, ?4)",
            (&self.run_id, workflow_name, goal, stencila_version),
        )?;
        Ok(())
    }

    /// Mark a run as completed with status, token count, and node count.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` on database failure.
    pub fn complete_run(
        &self,
        status: &str,
        total_tokens: i64,
        node_count: i64,
    ) -> Result<(), stencila_db::rusqlite::Error> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "UPDATE workflow_runs
             SET status = ?1,
                 total_tokens = ?2,
                 node_count = ?3,
                 total_duration_ms = CAST(
                    (julianday(strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) - julianday(started_at))
                    * 86400000.0 AS INTEGER
                 ),
                 completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE run_id = ?4",
            (status, total_tokens, node_count, &self.run_id),
        )?;
        Ok(())
    }

    /// Count node rows for this run.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` on database failure.
    pub fn node_count(&self) -> Result<i64, stencila_db::rusqlite::Error> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.query_row(
            "SELECT COUNT(*) FROM workflow_nodes WHERE run_id = ?1",
            (&self.run_id,),
            |row| row.get(0),
        )
    }

    /// Insert or update a node execution record.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` on database failure.
    pub fn upsert_node(&self, record: &NodeRecord<'_>) -> Result<(), stencila_db::rusqlite::Error> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "INSERT INTO workflow_nodes (run_id, node_id, status, model, provider, duration_ms,
                                input_tokens, output_tokens, retry_count, failure_reason,
                                started_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                     strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(run_id, node_id) DO UPDATE SET
                status = excluded.status,
                model = COALESCE(excluded.model, workflow_nodes.model),
                provider = COALESCE(excluded.provider, workflow_nodes.provider),
                duration_ms = COALESCE(excluded.duration_ms, workflow_nodes.duration_ms),
                input_tokens = COALESCE(excluded.input_tokens, workflow_nodes.input_tokens),
                output_tokens = COALESCE(excluded.output_tokens, workflow_nodes.output_tokens),
                retry_count = COALESCE(excluded.retry_count, workflow_nodes.retry_count),
                failure_reason = excluded.failure_reason,
                completed_at = CASE WHEN excluded.status IN ('completed', 'failed')
                               THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                               ELSE workflow_nodes.completed_at END",
            (
                &self.run_id,
                record.node_id,
                record.status,
                record.model,
                record.provider,
                record.duration_ms,
                record.input_tokens,
                record.output_tokens,
                record.retry_count,
                record.failure_reason,
            ),
        )?;
        Ok(())
    }

    /// Record an edge traversal.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` on database failure.
    pub fn insert_edge(
        &self,
        step_index: i64,
        from_node: &str,
        to_node: &str,
        edge_label: Option<&str>,
    ) -> Result<(), stencila_db::rusqlite::Error> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "INSERT INTO workflow_edges (run_id, step_index, from_node, to_node, edge_label)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (&self.run_id, step_index, from_node, to_node, edge_label),
        )?;
        Ok(())
    }

    /// Save a content-addressed definition snapshot (deduplicates by hash).
    ///
    /// `content` should be a zstd-compressed tar archive of the definition
    /// directory (created via [`crate::definition_snapshot::snapshot_dir`]).
    /// The `hash` is the SHA-256 hex digest of the compressed blob.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` on database failure.
    pub fn save_definition_snapshot(
        &self,
        hash: &str,
        content: &[u8],
        kind: &str,
        name: &str,
    ) -> Result<(), stencila_db::rusqlite::Error> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "INSERT OR IGNORE INTO workflow_definition_snapshots (content_hash, content, kind, name)
             VALUES (?1, ?2, ?3, ?4)",
            (hash, content, kind, name),
        )?;
        Ok(())
    }

    /// Link a definition snapshot to this run.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` on database failure.
    pub fn link_run_definition(
        &self,
        hash: &str,
        role: &str,
    ) -> Result<(), stencila_db::rusqlite::Error> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "INSERT OR IGNORE INTO workflow_run_definitions (run_id, content_hash, role)
             VALUES (?1, ?2, ?3)",
            (&self.run_id, hash, role),
        )?;
        Ok(())
    }

    /// Save a compressed LLM output for a node.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` on database failure.
    pub fn save_node_output(
        &self,
        node_id: &str,
        output: &[u8],
    ) -> Result<(), stencila_db::rusqlite::Error> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "INSERT INTO workflow_node_outputs (run_id, node_id, output)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(run_id, node_id) DO UPDATE SET output = excluded.output",
            (&self.run_id, node_id, output),
        )?;
        Ok(())
    }

    /// Register an artifact record for this run.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` on database failure.
    pub fn insert_artifact(
        &self,
        artifact_id: &str,
        name: &str,
        mime_type: Option<&str>,
        size_bytes: Option<i64>,
        path: &str,
    ) -> Result<(), stencila_db::rusqlite::Error> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "INSERT INTO workflow_artifacts (run_id, artifact_id, name, mime_type, size_bytes, path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (&self.run_id, artifact_id, name, mime_type, size_bytes, path),
        )?;
        Ok(())
    }

    /// Insert a completed interview record for this run.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` on database failure.
    #[allow(clippy::too_many_arguments)]
    pub fn insert_interview(
        &self,
        interview_id: &str,
        node_id: &str,
        question_text: &str,
        question_type: Option<&str>,
        options: Option<&str>,
        answer: Option<&str>,
        selected_option: Option<&str>,
        asked_at: &str,
        answered_at: Option<&str>,
        duration_ms: Option<i64>,
    ) -> Result<(), stencila_db::rusqlite::Error> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute(
            "INSERT INTO workflow_interviews (
                interview_id, run_id, node_id, question_text, question_type, options,
                answer, selected_option, asked_at, answered_at, duration_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            (
                interview_id,
                &self.run_id,
                node_id,
                question_text,
                question_type,
                options,
                answer,
                selected_option,
                asked_at,
                answered_at,
                duration_ms,
            ),
        )?;
        Ok(())
    }

    /// Delete this run and all run-scoped rows in a single transaction.
    ///
    /// Also removes `.stencila/artifacts/workflow-runs/{run_id}` when present.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` on database failure.
    pub fn delete_run(&self) -> Result<(), stencila_db::rusqlite::Error> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tx = conn.unchecked_transaction()?;

        // Child tables first, then parent workflow_runs row.
        for table in [
            "workflow_run_definitions",
            "workflow_context",
            "workflow_nodes",
            "workflow_artifacts",
            "workflow_logs",
            "workflow_edges",
            "workflow_interviews",
            "workflow_node_outputs",
        ] {
            let sql = format!("DELETE FROM {table} WHERE run_id = ?1");
            tx.execute(&sql, (&self.run_id,))?;
        }
        tx.execute(
            "DELETE FROM workflow_runs WHERE run_id = ?1",
            (&self.run_id,),
        )?;
        tx.commit()?;
        drop(conn);

        if let Some(path) = self.infer_artifacts_run_dir()
            && let Err(error) = std::fs::remove_dir_all(&path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(
                "Failed to remove run artifacts at `{}`: {error}",
                path.display()
            );
        }

        Ok(())
    }

    fn infer_artifacts_run_dir(&self) -> Option<PathBuf> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let db_path: String = conn
            .query_row("PRAGMA database_list", [], |row| row.get(2))
            .ok()?;
        let stencila_dir = std::path::Path::new(&db_path).parent()?;
        Some(
            stencila_dir
                .join("artifacts")
                .join("workflow-runs")
                .join(&self.run_id),
        )
    }
}

// ---------------------------------------------------------------------------
// ContextBackend implementation
// ---------------------------------------------------------------------------

impl ContextBackend for SqliteBackend {
    fn set(&self, key: &str, value: Value) {
        let value_str = value.to_string();
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Err(e) = conn.execute(
            "INSERT INTO workflow_context (run_id, key, value, updated_at)
             VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(run_id, key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at",
            (&self.run_id, key, &value_str),
        ) {
            tracing::warn!("SQLite context set({key}) failed: {e}");
        }
    }

    fn get(&self, key: &str) -> Option<Value> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.query_row(
            "SELECT value FROM workflow_context WHERE run_id = ?1 AND key = ?2",
            (&self.run_id, key),
            |row| {
                let s: String = row.get(0)?;
                Ok(serde_json::from_str(&s).unwrap_or(Value::String(s)))
            },
        )
        .ok()
    }

    fn snapshot(&self) -> IndexMap<String, Value> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Ok(mut stmt) = conn
            .prepare("SELECT key, value FROM workflow_context WHERE run_id = ?1 ORDER BY rowid")
        else {
            return IndexMap::new();
        };

        let Ok(rows) = stmt.query_map((&self.run_id,), |row| {
            let key: String = row.get(0)?;
            let val_str: String = row.get(1)?;
            let val = serde_json::from_str(&val_str).unwrap_or(Value::String(val_str));
            Ok((key, val))
        }) else {
            return IndexMap::new();
        };

        let mut map = IndexMap::new();
        for (k, v) in rows.flatten() {
            map.insert(k, v);
        }
        map
    }

    fn clone_backend(&self) -> Box<dyn ContextBackend> {
        // For parallel handler: snapshot into a writable in-memory backend.
        // Branches are ephemeral and merge results back via context_updates.
        Box::new(crate::context::InMemoryBackend::with_data(
            self.snapshot(),
            self.logs(),
        ))
    }

    fn apply_updates(&self, updates: &IndexMap<String, Value>) {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tx = match conn.unchecked_transaction() {
            Ok(tx) => tx,
            Err(e) => {
                tracing::warn!("SQLite apply_updates: failed to begin transaction: {e}");
                return;
            }
        };
        for (key, value) in updates {
            let value_str = value.to_string();
            if let Err(e) = tx.execute(
                "INSERT INTO workflow_context (run_id, key, value, updated_at)
                 VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                 ON CONFLICT(run_id, key) DO UPDATE SET
                    value = excluded.value,
                    updated_at = excluded.updated_at",
                (&self.run_id, key.as_str(), &value_str),
            ) {
                tracing::warn!("SQLite context apply_updates({key}) failed: {e}");
                return;
            }
        }
        if let Err(e) = tx.commit() {
            tracing::warn!("SQLite apply_updates: commit failed: {e}");
        }
    }

    fn append_log(&self, entry: &str) {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Err(e) = conn.execute(
            "INSERT INTO workflow_logs (run_id, entry) VALUES (?1, ?2)",
            (&self.run_id, entry),
        ) {
            tracing::warn!("SQLite append_log failed: {e}");
        }
    }

    fn logs(&self) -> Vec<String> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Ok(mut stmt) =
            conn.prepare("SELECT entry FROM workflow_logs WHERE run_id = ?1 ORDER BY timestamp")
        else {
            return Vec::new();
        };

        let Ok(rows) = stmt.query_map((&self.run_id,), |row| row.get::<_, String>(0)) else {
            return Vec::new();
        };

        rows.filter_map(Result::ok).collect()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
