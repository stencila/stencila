use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use stencila_db::rusqlite::{self, Connection, params};

use crate::error::{AgentError, AgentResult};
use crate::types::{SessionState, Turn, now_timestamp};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionPersistence {
    Persistent,
    Ephemeral,
    BestEffort,
    Required,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SmartDefault)]
pub struct PersistencePolicy {
    #[default(SessionPersistence::Ephemeral)]
    pub persistence: SessionPersistence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionOwner {
    pub user_id: Option<String>,
    pub workspace_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resumability {
    Full,
    None,
}

impl std::fmt::Display for Resumability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(resumability_to_str(self))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub backend_kind: String,
    pub agent_name: String,
    pub provider_name: String,
    pub model_name: String,
    pub state: SessionState,
    pub total_turns: i64,
    pub resumability: Resumability,
    pub created_at: String,
    pub updated_at: String,
    pub workflow_run_id: Option<String>,
    pub workflow_thread_id: Option<String>,
    pub workflow_node_id: Option<String>,
    pub provider_resume_state: Option<String>,
    pub config_snapshot: Option<String>,
    pub system_prompt: Option<String>,
    pub lease_holder: Option<String>,
    pub lease_expires_at: Option<String>,
}

/// Workflow attribution metadata for agent sessions created within a
/// workflow pipeline.
#[derive(Debug, Clone, Default)]
pub struct WorkflowAttribution {
    pub run_id: Option<String>,
    pub thread_id: Option<String>,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ListSessionsFilter {
    pub resumable: Option<bool>,
    pub workflow_run_id: Option<String>,
    pub workflow_thread_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateSession {
    pub state: SessionState,
    pub total_turns: i64,
    pub updated_at: String,
    pub resumability: Option<Resumability>,
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

const SESSION_COLUMNS: &str = "\
    session_id, backend_kind, agent_name, provider_name, model_name, \
    state, total_turns, resumability, created_at, updated_at, \
    workflow_run_id, workflow_thread_id, workflow_node_id, \
    provider_resume_state, config_snapshot, system_prompt, \
    lease_holder, lease_expires_at";

pub struct AgentSessionStore {
    conn: Arc<Mutex<Connection>>,
}

impl AgentSessionStore {
    #[must_use]
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Open (or create) the workspace database at `workspace_root/.stencila/db.sqlite3`,
    /// apply agent migrations, and return a ready-to-use store.
    pub fn open(workspace_root: &std::path::Path) -> Result<Self, String> {
        let db_path = workspace_root.join(".stencila").join("db.sqlite3");
        let db = stencila_db::WorkspaceDb::open(&db_path)
            .map_err(|e| format!("failed to open workspace DB: {e}"))?;
        db.migrate("agents", crate::migrations::AGENT_MIGRATIONS)
            .map_err(|e| format!("failed to apply agent migrations: {e}"))?;
        Ok(Self::new(db.connection().clone()))
    }

    fn lock_conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn insert_session(&self, record: &SessionRecord) -> Result<(), rusqlite::Error> {
        let conn = self.lock_conn();

        let state_str = state_to_str(record.state);
        let resumability_str = resumability_to_str(&record.resumability);

        conn.execute(
            "INSERT INTO agent_sessions (
                session_id, backend_kind, agent_name, provider_name, model_name,
                state, total_turns, resumability, created_at, updated_at,
                workflow_run_id, workflow_thread_id, workflow_node_id,
                provider_resume_state, config_snapshot, system_prompt,
                lease_holder, lease_expires_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                record.session_id,
                record.backend_kind,
                record.agent_name,
                record.provider_name,
                record.model_name,
                state_str,
                record.total_turns,
                resumability_str,
                record.created_at,
                record.updated_at,
                record.workflow_run_id,
                record.workflow_thread_id,
                record.workflow_node_id,
                record.provider_resume_state,
                record.config_snapshot,
                record.system_prompt,
                record.lease_holder,
                record.lease_expires_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_session(&self, session_id: &str) -> Result<Option<SessionRecord>, rusqlite::Error> {
        let conn = self.lock_conn();

        let sql = format!("SELECT {SESSION_COLUMNS} FROM agent_sessions WHERE session_id = ?1");
        let mut stmt = conn.prepare(&sql)?;

        let mut rows = stmt.query(params![session_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(record_from_row(row)?))
        } else {
            Ok(None)
        }
    }

    /// Look up a session by exact ID first, then by unique prefix.
    ///
    /// Returns `Ok(Some(record))` when exactly one session matches the
    /// prefix, `Ok(None)` when no sessions match, or an error when the
    /// prefix is ambiguous (matches more than one session).
    pub fn get_session_by_prefix(&self, prefix: &str) -> AgentResult<Option<SessionRecord>> {
        let db_err = |e: rusqlite::Error| AgentError::Io {
            message: format!("database error: {e}"),
        };

        if let Some(record) = self.get_session(prefix).map_err(db_err)? {
            return Ok(Some(record));
        }

        let conn = self.lock_conn();
        let like_pattern = format!("{prefix}%");
        let sql = format!("SELECT {SESSION_COLUMNS} FROM agent_sessions WHERE session_id LIKE ?1");
        let mut stmt = conn.prepare(&sql).map_err(db_err)?;
        let mut rows = stmt.query(params![like_pattern]).map_err(db_err)?;

        let first = match rows.next().map_err(db_err)? {
            Some(row) => record_from_row(row).map_err(db_err)?,
            None => return Ok(None),
        };

        if rows.next().map_err(db_err)?.is_some() {
            return Err(AgentError::InvalidState {
                expected: "unique session prefix".into(),
                actual: format!("ambiguous prefix '{prefix}' matches multiple sessions"),
            });
        }

        Ok(Some(first))
    }

    pub fn checkpoint_turns(
        &self,
        session_id: &str,
        turns: &[Turn],
    ) -> Result<(), rusqlite::Error> {
        let conn = self.lock_conn();

        conn.execute(
            "DELETE FROM agent_session_turns WHERE session_id = ?1",
            params![session_id],
        )?;

        for (i, turn) in turns.iter().enumerate() {
            let turn_json = serde_json::to_string(turn).unwrap_or_else(|_| "null".to_string());
            conn.execute(
                "INSERT INTO agent_session_turns (session_id, turn_index, turn_json) VALUES (?1, ?2, ?3)",
                params![session_id, i as i64, turn_json],
            )?;
        }

        Ok(())
    }

    pub fn get_turns(&self, session_id: &str) -> Result<Vec<Turn>, rusqlite::Error> {
        let conn = self.lock_conn();

        let mut stmt = conn.prepare(
            "SELECT turn_json FROM agent_session_turns
             WHERE session_id = ?1 ORDER BY turn_index ASC",
        )?;

        let rows: Result<Vec<String>, _> = stmt
            .query_map(params![session_id], |row| row.get(0))?
            .collect();

        let turns = rows?
            .iter()
            .filter_map(|json_str| serde_json::from_str::<Turn>(json_str).ok())
            .collect();

        Ok(turns)
    }

    pub fn list_sessions(
        &self,
        filter: &ListSessionsFilter,
    ) -> Result<Vec<SessionRecord>, rusqlite::Error> {
        let conn = self.lock_conn();

        let mut sql = format!("SELECT {SESSION_COLUMNS} FROM agent_sessions WHERE 1=1");

        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(true) = filter.resumable {
            param_values.push(Box::new("Full".to_string()));
            sql.push_str(&format!(" AND resumability = ?{}", param_values.len()));
        }

        if let Some(ref run_id) = filter.workflow_run_id {
            param_values.push(Box::new(run_id.clone()));
            sql.push_str(&format!(" AND workflow_run_id = ?{}", param_values.len()));
        }

        if let Some(ref thread_id) = filter.workflow_thread_id {
            param_values.push(Box::new(thread_id.clone()));
            sql.push_str(&format!(
                " AND workflow_thread_id = ?{}",
                param_values.len()
            ));
        }

        sql.push_str(" ORDER BY updated_at DESC, session_id DESC");

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|b| b.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_ref.as_slice(), record_from_row)?;

        rows.collect()
    }

    pub fn update_session(
        &self,
        session_id: &str,
        update: &UpdateSession,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.lock_conn();

        let state_str = state_to_str(update.state);
        let mut set_clauses = vec![
            "state = ?1".to_string(),
            "total_turns = ?2".to_string(),
            "updated_at = ?3".to_string(),
        ];
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
            Box::new(state_str.to_string()),
            Box::new(update.total_turns),
            Box::new(update.updated_at.clone()),
        ];

        if let Some(ref resumability) = update.resumability {
            param_values.push(Box::new(resumability_to_str(resumability).to_string()));
            set_clauses.push(format!("resumability = ?{}", param_values.len()));
        }

        param_values.push(Box::new(session_id.to_string()));
        let where_idx = param_values.len();
        let sql = format!(
            "UPDATE agent_sessions SET {} WHERE session_id = ?{where_idx}",
            set_clauses.join(", ")
        );

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|b| b.as_ref()).collect();
        conn.execute(&sql, params_ref.as_slice())?;

        Ok(())
    }

    // -- Checkpoint ---------------------------------------------------------

    pub fn upsert_checkpoint(
        &self,
        record: &SessionRecord,
        turns: &[Turn],
    ) -> Result<(), rusqlite::Error> {
        match self.get_session(&record.session_id)? {
            Some(_) => {
                let update = UpdateSession {
                    state: record.state,
                    total_turns: record.total_turns,
                    updated_at: now_timestamp(),
                    resumability: Some(record.resumability.clone()),
                };
                self.update_session(&record.session_id, &update)?;
                if let Some(ref resume_state) = record.provider_resume_state {
                    self.set_provider_resume_state(&record.session_id, Some(resume_state))?;
                }
            }
            None => {
                self.insert_session(record)?;
            }
        }
        self.checkpoint_turns(&record.session_id, turns)?;
        Ok(())
    }

    // -- Resume state -------------------------------------------------------

    pub fn set_provider_resume_state(
        &self,
        session_id: &str,
        resume_state: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.lock_conn();
        conn.execute(
            "UPDATE agent_sessions SET provider_resume_state = ?1 WHERE session_id = ?2",
            params![resume_state, session_id],
        )?;
        Ok(())
    }

    // -- Lease management ---------------------------------------------------

    pub fn acquire_lease(
        &self,
        session_id: &str,
        holder: &str,
        expires_at: &str,
    ) -> Result<bool, rusqlite::Error> {
        let conn = self.lock_conn();
        let now = now_timestamp();

        // Atomically set lease if:
        //   - no lease holder exists (NULL), OR
        //   - the same holder already holds it, OR
        //   - the lease has expired
        let rows_updated = conn.execute(
            "UPDATE agent_sessions
             SET lease_holder = ?1, lease_expires_at = ?2
             WHERE session_id = ?3
               AND (lease_holder IS NULL
                    OR lease_holder = ?1
                    OR lease_expires_at <= ?4)",
            params![holder, expires_at, session_id, now],
        )?;

        Ok(rows_updated > 0)
    }

    pub fn renew_lease(
        &self,
        session_id: &str,
        holder: &str,
        new_expires_at: &str,
    ) -> Result<bool, rusqlite::Error> {
        let conn = self.lock_conn();

        let rows_updated = conn.execute(
            "UPDATE agent_sessions
             SET lease_expires_at = ?1
             WHERE session_id = ?2
               AND lease_holder = ?3",
            params![new_expires_at, session_id, holder],
        )?;

        Ok(rows_updated > 0)
    }

    pub fn release_lease(&self, session_id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.lock_conn();

        conn.execute(
            "UPDATE agent_sessions
             SET lease_holder = NULL, lease_expires_at = NULL
             WHERE session_id = ?1",
            params![session_id],
        )?;

        Ok(())
    }

    // -- Resume queries -----------------------------------------------------

    pub fn find_latest_resumable_standalone(
        &self,
    ) -> Result<Option<SessionRecord>, rusqlite::Error> {
        let conn = self.lock_conn();

        let sql = format!(
            "SELECT {SESSION_COLUMNS} FROM agent_sessions
             WHERE resumability = ?1
               AND state != ?2
               AND workflow_run_id IS NULL
             ORDER BY updated_at DESC, session_id DESC
             LIMIT 1"
        );

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(params![
            resumability_to_str(&Resumability::Full),
            state_to_str(SessionState::Closed),
        ])?;

        if let Some(row) = rows.next()? {
            Ok(Some(record_from_row(row)?))
        } else {
            Ok(None)
        }
    }

    /// Resolve workflow names for a set of run IDs.
    ///
    /// Queries the `workflow_runs` table (managed by the workflows crate)
    /// and returns a map from `run_id` to `workflow_name`. Missing or
    /// inaccessible rows are silently omitted.
    pub fn resolve_workflow_names(
        &self,
        run_ids: &[&str],
    ) -> std::collections::HashMap<String, String> {
        let mut result = std::collections::HashMap::new();
        if run_ids.is_empty() {
            return result;
        }

        let conn = self.lock_conn();

        for run_id in run_ids {
            if let Ok(name) = conn.query_row(
                "SELECT workflow_name FROM workflow_runs WHERE run_id = ?1",
                params![run_id],
                |row| row.get::<_, String>(0),
            ) {
                result.insert((*run_id).to_string(), name);
            }
        }

        result
    }
}

// ---------------------------------------------------------------------------
// Shared persistence helpers
// ---------------------------------------------------------------------------

/// Check whether the persistence mode requires writing checkpoints.
pub(crate) fn should_persist(mode: Option<&SessionPersistence>) -> bool {
    !matches!(mode, None | Some(SessionPersistence::Ephemeral))
}

/// Handle a checkpoint result according to the persistence policy.
///
/// - [`Required`](SessionPersistence::Required): propagate the error
/// - [`BestEffort`](SessionPersistence::BestEffort): log a warning and return `Ok(())`
/// - Other modes: swallow the error silently
pub(crate) fn handle_checkpoint_result(
    mode: Option<&SessionPersistence>,
    result: AgentResult<()>,
) -> AgentResult<()> {
    match result {
        Ok(()) => Ok(()),
        Err(e) => match mode {
            Some(SessionPersistence::Required) => Err(e),
            Some(SessionPersistence::BestEffort) => {
                tracing::warn!("checkpoint failed (best-effort): {e}");
                Ok(())
            }
            _ => Ok(()),
        },
    }
}

/// Write a checkpoint to the store, mapping database errors to [`AgentError::Io`].
pub(crate) fn write_checkpoint(
    store: &AgentSessionStore,
    record: &SessionRecord,
    history: &[Turn],
) -> AgentResult<()> {
    store
        .upsert_checkpoint(record, history)
        .map_err(|e| AgentError::Io {
            message: format!("checkpoint write failed: {e}"),
        })
}

/// Extract workflow attribution fields from an optional [`WorkflowAttribution`].
pub(crate) fn workflow_fields(
    attribution: Option<&WorkflowAttribution>,
) -> (Option<String>, Option<String>, Option<String>) {
    let run_id = attribution.and_then(|w| w.run_id.clone());
    let thread_id = attribution.and_then(|w| w.thread_id.clone());
    let node_id = attribution.and_then(|w| w.node_id.clone());
    (run_id, thread_id, node_id)
}

// ---------------------------------------------------------------------------
// Row mapping
// ---------------------------------------------------------------------------

fn record_from_row(row: &rusqlite::Row<'_>) -> Result<SessionRecord, rusqlite::Error> {
    let state = state_from_str(&row.get::<_, String>(5)?);
    let resumability = resumability_from_str(&row.get::<_, String>(7)?);
    Ok(SessionRecord {
        session_id: row.get(0)?,
        backend_kind: row.get(1)?,
        agent_name: row.get(2)?,
        provider_name: row.get(3)?,
        model_name: row.get(4)?,
        state,
        total_turns: row.get(6)?,
        resumability,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        workflow_run_id: row.get(10)?,
        workflow_thread_id: row.get(11)?,
        workflow_node_id: row.get(12)?,
        provider_resume_state: row.get(13)?,
        config_snapshot: row.get(14)?,
        system_prompt: row.get(15)?,
        lease_holder: row.get(16)?,
        lease_expires_at: row.get(17)?,
    })
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

fn state_to_str(state: SessionState) -> &'static str {
    match state {
        SessionState::Idle => "IDLE",
        SessionState::Processing => "PROCESSING",
        SessionState::AwaitingInput => "AWAITING_INPUT",
        SessionState::Closed => "CLOSED",
    }
}

fn state_from_str(s: &str) -> SessionState {
    match s {
        "IDLE" => SessionState::Idle,
        "PROCESSING" => SessionState::Processing,
        "AWAITING_INPUT" => SessionState::AwaitingInput,
        "CLOSED" => SessionState::Closed,
        _ => SessionState::default(),
    }
}

fn resumability_to_str(r: &Resumability) -> &'static str {
    match r {
        Resumability::Full => "Full",
        Resumability::None => "None",
    }
}

fn resumability_from_str(s: &str) -> Resumability {
    match s {
        "Full" => Resumability::Full,
        _ => Resumability::None,
    }
}

// ---------------------------------------------------------------------------
// Resume validation
// ---------------------------------------------------------------------------

/// Validate that a session can be resumed by `caller_holder`, and load it.
///
/// When `session_id` is `None`, the latest resumable standalone session is
/// selected automatically. When `Some`, that specific session is loaded and
/// checked.
///
/// Validates:
/// - The session exists
/// - The session is not closed
/// - The session is not workflow-owned
/// - The session is resumable
/// - No other process holds an active (non-expired) lease
///
/// On success returns the [`SessionRecord`] and its persisted turn history.
pub fn validate_session_for_resume(
    store: &AgentSessionStore,
    session_id: Option<&str>,
    caller_holder: &str,
) -> AgentResult<(SessionRecord, Vec<Turn>)> {
    let db_err = |e: rusqlite::Error| AgentError::Io {
        message: format!("database error: {e}"),
    };

    let record = match session_id {
        Some(id) => {
            store
                .get_session(id)
                .map_err(db_err)?
                .ok_or_else(|| AgentError::InvalidState {
                    expected: "existing session".into(),
                    actual: format!("session '{id}' not found"),
                })?
        }
        None => store
            .find_latest_resumable_standalone()
            .map_err(db_err)?
            .ok_or_else(|| AgentError::InvalidState {
                expected: "resumable session".into(),
                actual: "no resumable sessions found".into(),
            })?,
    };

    // Reject closed sessions
    if record.state == SessionState::Closed {
        return Err(AgentError::InvalidState {
            expected: "non-closed session".into(),
            actual: "session is closed".into(),
        });
    }

    // Reject workflow-owned sessions
    if record.workflow_run_id.is_some() {
        return Err(AgentError::InvalidState {
            expected: "standalone session".into(),
            actual: "session is owned by a workflow".into(),
        });
    }

    // Reject non-resumable sessions
    if record.resumability != Resumability::Full {
        return Err(AgentError::InvalidState {
            expected: "resumable session".into(),
            actual: "session is not resumable".into(),
        });
    }

    // Check lease conflict: reject if a different holder has a non-expired lease
    if let Some(ref holder) = record.lease_holder
        && holder != caller_holder
    {
        let lease_active = record
            .lease_expires_at
            .as_ref()
            .is_some_and(|exp| exp > &now_timestamp());
        if lease_active {
            return Err(AgentError::LeaseConflict {
                holder: holder.clone(),
                session_id: record.session_id.clone(),
            });
        }
    }

    let turns = store.get_turns(&record.session_id).map_err(db_err)?;

    Ok((record, turns))
}
