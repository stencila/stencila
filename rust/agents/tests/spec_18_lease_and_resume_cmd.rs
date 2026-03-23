//! Phase 4 / Slices 4-5 tests: Lease-conflict detection in `resume_session`
//! and CLI `stencila agents sessions resume` command validation logic.
//!
//! Acceptance criteria tested:
//!
//! - AC-S4-1: Acquire lease with holder "process-A", attempt resume from
//!   "process-B" → explicit concurrency error returned.
//! - AC-S4-2: Same holder can resume without conflict.
//! - AC-S4-3: Expired lease allows resume from different holder.
//! - AC-S5-1: `validate_session_for_resume` selects latest resumable
//!   standalone session when no session-id is given.
//! - AC-S5-2: `validate_session_for_resume` rejects closed sessions.
//! - AC-S5-3: `validate_session_for_resume` rejects workflow-owned sessions.
//! - AC-S5-extra: Non-resumable and nonexistent sessions are rejected.
//!
//! Non-test deliverables:
//! - `SessionsCommand::Resume` wiring: verified by `cargo check -p stencila-cli`
//!   (the `cli` module is gated behind `feature = "cli"` which is only enabled
//!   by the CLI crate).
//! - Outside-workspace error message: integration-level concern, not unit-testable.
//! - `cargo clippy`, `cargo fmt`, `cargo test` pass: CI obligation.
//!
//! All tests use in-memory SQLite and mock infrastructure.

#![allow(clippy::result_large_err)]

use std::sync::{Arc, Mutex};

use stencila_agents::error::AgentError;
use stencila_agents::migrations::AGENT_MIGRATIONS;
use stencila_agents::store::{
    AgentSessionStore,
    Resumability,
    SessionRecord,
    // New function that checks lease + validates session for resume:
    validate_session_for_resume,
};
use stencila_agents::types::SessionState;

// ===========================================================================
// Test infrastructure
// ===========================================================================

fn setup_db() -> Arc<Mutex<stencila_db::rusqlite::Connection>> {
    let conn = stencila_db::rusqlite::Connection::open_in_memory().expect("open in-memory SQLite");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("enable FK");
    for m in AGENT_MIGRATIONS {
        conn.execute_batch(m.sql).expect("apply migration");
    }
    Arc::new(Mutex::new(conn))
}

fn sample_record(session_id: &str) -> SessionRecord {
    SessionRecord {
        session_id: session_id.to_string(),
        backend_kind: "api".to_string(),
        agent_name: "general".to_string(),
        provider_name: "test".to_string(),
        model_name: "test-model".to_string(),
        state: SessionState::Idle,
        total_turns: 0,
        resumability: Resumability::Full,
        created_at: "2025-07-01T00:00:00Z".to_string(),
        updated_at: "2025-07-01T00:00:00Z".to_string(),
        workflow_run_id: None,
        workflow_thread_id: None,
        workflow_node_id: None,
        provider_resume_state: None,
        config_snapshot: None,
        system_prompt: None,
        lease_holder: None,
        lease_expires_at: None,
    }
}

/// Lease expiry far in the future (ensures the lease is valid during tests).
fn future_expiry() -> String {
    "2099-12-31T23:59:59Z".to_string()
}

/// Lease expiry in the past (ensures the lease is expired during tests).
fn past_expiry() -> String {
    "2020-01-01T00:00:00Z".to_string()
}

// ===========================================================================
// AC-S4-1: Lease conflict — process-B blocked by process-A's active lease
// ===========================================================================

#[test]
fn resume_with_active_lease_from_different_holder_returns_lease_conflict() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    // Insert a resumable session
    let record = sample_record("sess-leased");
    store.insert_session(&record).expect("insert session");

    // Process-A acquires the lease
    let acquired = store
        .acquire_lease("sess-leased", "process-A", &future_expiry())
        .expect("acquire lease");
    assert!(acquired, "process-A should acquire the lease");

    // Process-B attempts to validate/resume → should get a lease-conflict error
    let result = validate_session_for_resume(&store, Some("sess-leased"), "process-B");

    assert!(result.is_err(), "should fail with lease conflict");
    let err = result.expect_err("should fail with lease conflict");
    match err {
        AgentError::LeaseConflict { holder, session_id } => {
            assert_eq!(holder, "process-A");
            assert_eq!(session_id, "sess-leased");
        }
        other => panic!("expected AgentError::LeaseConflict, got: {other:?}"),
    }
}

// ===========================================================================
// AC-S4-2: Same holder can resume without conflict
// ===========================================================================

#[test]
fn resume_with_active_lease_from_same_holder_succeeds() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    let record = sample_record("sess-same-holder");
    store.insert_session(&record).expect("insert session");

    // Process-A acquires the lease
    let acquired = store
        .acquire_lease("sess-same-holder", "process-A", &future_expiry())
        .expect("acquire lease");
    assert!(acquired);

    // Process-A attempts to resume — should succeed (same holder)
    let result = validate_session_for_resume(&store, Some("sess-same-holder"), "process-A");

    assert!(
        result.is_ok(),
        "same holder should be able to resume: {result:?}"
    );
}

// ===========================================================================
// AC-S4-3: Expired lease allows resume from different holder
// ===========================================================================

#[test]
fn resume_with_expired_lease_from_different_holder_succeeds() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    let record = sample_record("sess-expired-lease");
    store.insert_session(&record).expect("insert session");

    // Process-A acquires a lease that is already expired
    let acquired = store
        .acquire_lease("sess-expired-lease", "process-A", &past_expiry())
        .expect("acquire lease");
    assert!(acquired);

    // Process-B attempts to resume — should succeed because lease is expired
    let result = validate_session_for_resume(&store, Some("sess-expired-lease"), "process-B");

    assert!(
        result.is_ok(),
        "expired lease should allow resume from different holder: {result:?}"
    );
}

// ===========================================================================
// AC-S5-1: Latest resumable standalone selected when no ID given
// ===========================================================================

#[test]
fn validate_resume_selects_latest_standalone_when_no_id() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    // Insert two resumable standalone sessions
    let mut old = sample_record("sess-old");
    old.updated_at = "2025-07-01T00:00:00Z".to_string();
    store.insert_session(&old).expect("insert old");

    let mut newer = sample_record("sess-newer");
    newer.updated_at = "2025-07-01T01:00:00Z".to_string();
    store.insert_session(&newer).expect("insert newer");

    // No session-id given → should select the newest
    let result = validate_session_for_resume(
        &store,
        None, // no session-id
        "process-X",
    );

    let (record, _turns) = result.expect("should succeed with latest session");
    assert_eq!(
        record.session_id, "sess-newer",
        "should select the newest resumable standalone session"
    );
}

#[test]
fn validate_resume_returns_error_when_no_sessions_exist() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    // No sessions at all
    let result = validate_session_for_resume(&store, None, "process-X");

    assert!(
        result.is_err(),
        "should fail when no resumable sessions exist"
    );
}

// ===========================================================================
// AC-S5-2: Closed sessions are rejected
// ===========================================================================

#[test]
fn validate_resume_rejects_closed_session() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    let mut closed = sample_record("sess-closed");
    closed.state = SessionState::Closed;
    closed.resumability = Resumability::Full;
    store.insert_session(&closed).expect("insert closed");

    // Explicit ID pointing to a closed session → should fail
    let result = validate_session_for_resume(&store, Some("sess-closed"), "process-X");

    assert!(result.is_err(), "should reject closed sessions");
    let err_msg = result
        .expect_err("should reject closed sessions")
        .to_string();
    // Error message should mention "closed"
    assert!(
        err_msg.to_lowercase().contains("closed"),
        "error message should mention 'closed', got: {err_msg}"
    );
}

// ===========================================================================
// AC-S5-3: Workflow-owned sessions are rejected
// ===========================================================================

#[test]
fn validate_resume_rejects_workflow_owned_session() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    let mut wf_owned = sample_record("sess-wf-owned");
    wf_owned.workflow_run_id = Some("wf-run-1".to_string());
    wf_owned.resumability = Resumability::Full;
    store.insert_session(&wf_owned).expect("insert workflow");

    // Explicit ID pointing to a workflow-owned session → should fail
    let result = validate_session_for_resume(&store, Some("sess-wf-owned"), "process-X");

    assert!(result.is_err(), "should reject workflow-owned sessions");
    let err_msg = result
        .expect_err("should reject workflow-owned sessions")
        .to_string();
    // Error message should mention "workflow"
    assert!(
        err_msg.to_lowercase().contains("workflow"),
        "error message should mention 'workflow', got: {err_msg}"
    );
}

// ===========================================================================
// AC-S5-extra: Non-resumable session is rejected
// ===========================================================================

#[test]
fn validate_resume_rejects_non_resumable_session() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    let mut non_res = sample_record("sess-no-resume");
    non_res.resumability = Resumability::None;
    store.insert_session(&non_res).expect("insert");

    let result = validate_session_for_resume(&store, Some("sess-no-resume"), "process-X");

    assert!(result.is_err(), "should reject non-resumable sessions");
}

// ===========================================================================
// AC-S5-extra: Nonexistent session ID returns clear error
// ===========================================================================

#[test]
fn validate_resume_rejects_nonexistent_session_id() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    let result = validate_session_for_resume(&store, Some("does-not-exist"), "process-X");

    assert!(result.is_err(), "should fail for nonexistent session ID");
    let err_msg = result
        .expect_err("should fail for nonexistent session ID")
        .to_string();
    assert!(
        err_msg.to_lowercase().contains("not found"),
        "error message should mention 'not found', got: {err_msg}"
    );
}
