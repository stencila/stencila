//! Audit database: migration, event writer, and background task.
//!
//! Records `Warn` and `Deny` guard verdicts to a SQLite database for
//! post-hoc review. `Allow` verdicts are never recorded.
//!
//! The writer runs as a background tokio task consuming events from a bounded
//! `mpsc` channel. If the database cannot be opened or migrated, auditing is
//! silently disabled — guard enforcement still runs.

#[cfg(feature = "tool-guard")]
use std::path::Path;

#[cfg(feature = "tool-guard")]
use tokio::sync::mpsc;

#[cfg(feature = "tool-guard")]
use crate::migrations::AGENT_MIGRATIONS;

/// Audit channel buffer size. Events beyond this are dropped (best-effort).
#[cfg(feature = "tool-guard")]
const CHANNEL_CAPACITY: usize = 256;

/// A single audit event to be written to the database.
#[cfg(feature = "tool-guard")]
#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub session_id: String,
    pub agent_name: String,
    pub trust_level: String,
    pub tool_name: String,
    pub input: String,
    pub matched_segment: String,
    pub verdict: &'static str,
    pub rule_id: &'static str,
    pub reason: &'static str,
    pub suggestion: &'static str,
}

/// Handle for sending audit events to the background writer.
///
/// Cheaply cloneable. Sending is non-blocking and best-effort: if the
/// channel is full or the receiver has been dropped, the event is silently
/// discarded.
#[cfg(feature = "tool-guard")]
#[derive(Debug, Clone)]
pub struct AuditSender {
    tx: mpsc::Sender<AuditEvent>,
}

#[cfg(feature = "tool-guard")]
impl AuditSender {
    /// Send an audit event (best-effort, non-blocking).
    pub fn send(&self, event: AuditEvent) {
        // Use try_send to avoid blocking the guard evaluation path.
        // If the channel is full or closed, we silently drop the event.
        let _ = self.tx.try_send(event);
    }
}

/// Spawn the audit background writer and return a sender handle.
///
/// Opens (or creates) the workspace database at
/// `<workspace_root>/.stencila/db.sqlite3`, applies migrations, and starts
/// a background task that drains the channel and inserts rows.
///
/// If the database cannot be opened or migrated, returns `None` and logs a
/// warning. Guard enforcement is unaffected.
#[cfg(feature = "tool-guard")]
pub fn spawn_audit_writer(workspace_root: &Path) -> Option<AuditSender> {
    let db_dir = workspace_root.join(".stencila");
    if let Err(e) = std::fs::create_dir_all(&db_dir) {
        tracing::warn!(
            "Tool guard audit disabled: cannot create .stencila directory at {}: {e}",
            db_dir.display()
        );
        return None;
    }

    let db_path = db_dir.join("db.sqlite3");
    let db = match stencila_db::WorkspaceDb::open(&db_path) {
        Ok(db) => db,
        Err(e) => {
            tracing::warn!(
                "Tool guard audit disabled: cannot open database at {}: {e}",
                db_path.display()
            );
            return None;
        }
    };

    if let Err(e) = db.migrate("agents", AGENT_MIGRATIONS) {
        tracing::warn!("Tool guard audit disabled: migration failed: {e}");
        return None;
    }

    let conn = db.connection().clone();
    let (tx, rx) = mpsc::channel(CHANNEL_CAPACITY);

    // Spawn requires a tokio runtime. If none is available (e.g. in sync
    // tests), audit is silently disabled.
    let handle = match tokio::runtime::Handle::try_current() {
        Ok(h) => h,
        Err(_) => {
            tracing::debug!("Tool guard audit disabled: no tokio runtime available");
            return None;
        }
    };
    handle.spawn(audit_writer_task(conn, rx));

    Some(AuditSender { tx })
}

#[cfg(feature = "tool-guard")]
async fn audit_writer_task(
    conn: std::sync::Arc<std::sync::Mutex<stencila_db::rusqlite::Connection>>,
    mut rx: mpsc::Receiver<AuditEvent>,
) {
    while let Some(event) = rx.recv().await {
        // Perform the blocking DB insert on the current task. The audit writer
        // is the only consumer and inserts are fast (single row, WAL mode).
        let conn = conn.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Err(e) = conn.execute(
            "INSERT INTO agent_tool_guard_events \
             (session_id, agent_name, trust_level, tool_name, input, \
              matched_segment, verdict, rule_id, reason, suggestion) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            (
                &event.session_id,
                &event.agent_name,
                &event.trust_level,
                &event.tool_name,
                &event.input,
                &event.matched_segment,
                event.verdict,
                event.rule_id,
                event.reason,
                event.suggestion,
            ),
        ) {
            tracing::warn!("Tool guard audit write failed: {e}");
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "tool-guard"))]
mod tests {
    use super::*;

    /// Helper: open an in-memory-style temp DB for testing.
    fn test_db(dir: &std::path::Path) -> stencila_db::WorkspaceDb {
        std::fs::create_dir_all(dir.join(".stencila")).unwrap();
        let db_path = dir.join(".stencila/db.sqlite3");
        stencila_db::WorkspaceDb::open(&db_path).unwrap()
    }

    fn sample_event(verdict: &'static str, rule_id: &'static str) -> AuditEvent {
        AuditEvent {
            session_id: "test-session".into(),
            agent_name: "test-agent".into(),
            trust_level: "medium".into(),
            tool_name: "shell".into(),
            input: "rm -rf /".into(),
            matched_segment: "rm -rf /".into(),
            verdict,
            rule_id,
            reason: "Recursive delete of root",
            suggestion: "Remove the path or use a safer command.",
        }
    }

    #[tokio::test]
    async fn audit_event_written_for_deny() {
        let tmp = tempfile::tempdir().unwrap();
        let db = test_db(tmp.path());
        db.migrate("agents", AGENT_MIGRATIONS).unwrap();

        let conn = db.connection().clone();
        let (tx, rx) = mpsc::channel(16);

        let writer_conn = conn.clone();
        let handle = tokio::spawn(audit_writer_task(writer_conn, rx));

        tx.send(sample_event("Deny", "core.recursive_delete_root"))
            .await
            .unwrap();
        drop(tx); // Close channel so writer exits
        handle.await.unwrap();

        let conn = conn.lock().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM agent_tool_guard_events WHERE verdict = 'Deny'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let rule: String = conn
            .query_row(
                "SELECT rule_id FROM agent_tool_guard_events WHERE verdict = 'Deny'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(rule, "core.recursive_delete_root");
    }

    #[tokio::test]
    async fn audit_event_written_for_warn() {
        let tmp = tempfile::tempdir().unwrap();
        let db = test_db(tmp.path());
        db.migrate("agents", AGENT_MIGRATIONS).unwrap();

        let conn = db.connection().clone();
        let (tx, rx) = mpsc::channel(16);

        let writer_conn = conn.clone();
        let handle = tokio::spawn(audit_writer_task(writer_conn, rx));

        tx.send(sample_event("Warn", "web.non_https"))
            .await
            .unwrap();
        drop(tx);
        handle.await.unwrap();

        let conn = conn.lock().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM agent_tool_guard_events WHERE verdict = 'Warn'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn no_audit_event_for_allow() {
        let tmp = tempfile::tempdir().unwrap();
        let db = test_db(tmp.path());
        db.migrate("agents", AGENT_MIGRATIONS).unwrap();

        let conn = db.connection().clone();
        // No events sent, table should be empty
        let conn = conn.lock().unwrap();
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM agent_tool_guard_events", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn multi_target_correct_decisive_path() {
        let tmp = tempfile::tempdir().unwrap();
        let db = test_db(tmp.path());
        db.migrate("agents", AGENT_MIGRATIONS).unwrap();

        let conn = db.connection().clone();
        let (tx, rx) = mpsc::channel(16);

        let writer_conn = conn.clone();
        let handle = tokio::spawn(audit_writer_task(writer_conn, rx));

        // Simulate a read_many_files audit with JSON array input and decisive path
        let event = AuditEvent {
            session_id: "sess-1".into(),
            agent_name: "agent-1".into(),
            trust_level: "medium".into(),
            tool_name: "read_many_files".into(),
            input: r#"["/workspace/ok.rs","/etc/shadow"]"#.into(),
            matched_segment: "/etc/shadow".into(),
            verdict: "Deny",
            rule_id: "file.system_path_read",
            reason: "System path",
            suggestion: "Use a workspace-local file.",
        };
        tx.send(event).await.unwrap();
        drop(tx);
        handle.await.unwrap();

        let conn = conn.lock().unwrap();
        let (input, segment): (String, String) = conn
            .query_row(
                "SELECT input, matched_segment FROM agent_tool_guard_events WHERE tool_name = 'read_many_files'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert!(input.contains("/etc/shadow"));
        assert_eq!(segment, "/etc/shadow");
    }

    #[tokio::test]
    async fn db_failure_does_not_block_guard() {
        // Create a temp dir, then place a regular file where .stencila/db.sqlite3
        // would go, making the directory creation fail deterministically.
        let tmp = tempfile::tempdir().unwrap();
        let blocker = tmp.path().join(".stencila");
        std::fs::write(&blocker, b"not a directory").unwrap();
        let result = spawn_audit_writer(tmp.path());
        assert!(result.is_none(), "Audit should be disabled when DB cannot open");
    }

    #[tokio::test]
    async fn spawn_audit_writer_creates_db_and_writes() {
        let tmp = tempfile::tempdir().unwrap();
        let sender = spawn_audit_writer(tmp.path()).expect("should create audit writer");

        sender.send(sample_event("Deny", "core.recursive_delete_root"));

        // Give the background task time to process
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        drop(sender);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Verify the event was written
        let db_path = tmp.path().join(".stencila/db.sqlite3");
        let db = stencila_db::WorkspaceDb::open(&db_path).unwrap();
        let conn = db.connection().lock().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM agent_tool_guard_events WHERE verdict = 'Deny'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
