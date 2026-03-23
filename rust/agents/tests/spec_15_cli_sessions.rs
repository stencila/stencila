//! Tests for Phase 3 / Slice 4: CLI sessions list/show commands
//! and persistence wiring in `stencila agents run`.
//!
//! These tests cover:
//! - Output formatting for `sessions list` (table row formatting)
//! - Output formatting for `sessions show` (detail display)
//! - `--resumable` filtering flag
//! - `agents sessions` subcommand group wired into the CLI parser
//! - Persistence wiring in `stencila agents run` (functions that open
//!   workspace DB, apply migrations, create store)
//!
//! Most tests require the `cli` feature. The store-level filtering tests
//! work without it.

#![allow(clippy::result_large_err)]

use std::sync::{Arc, Mutex};

use stencila_agents::migrations::AGENT_MIGRATIONS;
use stencila_agents::store::{AgentSessionStore, ListSessionsFilter, Resumability, SessionRecord};
use stencila_agents::types::SessionState;

// ===========================================================================
// Helpers (reused from spec_11_store pattern)
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

fn make_store(conn: &Arc<Mutex<stencila_db::rusqlite::Connection>>) -> AgentSessionStore {
    AgentSessionStore::new(conn.clone())
}

fn sample_record(session_id: &str) -> SessionRecord {
    SessionRecord {
        session_id: session_id.to_string(),
        backend_kind: "api".to_string(),
        agent_name: "general".to_string(),
        provider_name: "anthropic".to_string(),
        model_name: "claude-sonnet-4-20250514".to_string(),
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

// ===========================================================================
// AC-1: format_session_list_row produces expected table columns
// ===========================================================================

/// `format_session_list_row` should return a struct/tuple with the columns
/// needed for the sessions list table: truncated ID, agent name, workflow,
/// state, resumability, total turns, updated time.
#[cfg(feature = "cli")]
#[test]
fn format_session_list_row_includes_truncated_id() {
    let record = SessionRecord {
        session_id: "abcdef12-3456-7890-abcd-ef1234567890".to_string(),
        ..sample_record("ignored")
    };
    let wf = std::collections::HashMap::new();

    let row = stencila_agents::cli::format_session_list_row(&record, &wf);

    // Session ID should be truncated (first 8 chars or similar short form)
    assert!(
        row.id.len() < record.session_id.len(),
        "ID should be truncated; got '{}'",
        row.id
    );
    assert!(
        record.session_id.starts_with(&row.id),
        "truncated ID should be a prefix of the full ID"
    );
}

#[cfg(feature = "cli")]
#[test]
fn format_session_list_row_includes_agent_name() {
    let mut record = sample_record("sess-1");
    record.agent_name = "code-engineer".to_string();
    let wf = std::collections::HashMap::new();

    let row = stencila_agents::cli::format_session_list_row(&record, &wf);
    assert_eq!(row.agent_name, "code-engineer");
}

#[cfg(feature = "cli")]
#[test]
fn format_session_list_row_includes_workflow_name() {
    let mut record = sample_record("sess-1");
    record.workflow_run_id = Some("run-123".to_string());
    let mut wf = std::collections::HashMap::new();
    wf.insert("run-123".to_string(), "my-workflow".to_string());

    let row = stencila_agents::cli::format_session_list_row(&record, &wf);
    assert_eq!(row.workflow, "my-workflow");
}

#[cfg(feature = "cli")]
#[test]
fn format_session_list_row_workflow_empty_for_standalone() {
    let record = sample_record("sess-1");
    let wf = std::collections::HashMap::new();

    let row = stencila_agents::cli::format_session_list_row(&record, &wf);
    assert_eq!(row.workflow, "");
}

#[cfg(feature = "cli")]
#[test]
fn format_session_list_row_includes_state() {
    let mut record = sample_record("sess-1");
    record.state = SessionState::AwaitingInput;
    let wf = std::collections::HashMap::new();

    let row = stencila_agents::cli::format_session_list_row(&record, &wf);
    // State should be a human-readable string
    assert!(!row.state.is_empty(), "state column should not be empty");
}

#[cfg(feature = "cli")]
#[test]
fn format_session_list_row_includes_resumability() {
    let mut record = sample_record("sess-1");
    record.resumability = Resumability::Full;
    let wf = std::collections::HashMap::new();

    let row = stencila_agents::cli::format_session_list_row(&record, &wf);
    assert!(
        !row.resumability.is_empty(),
        "resumability column should not be empty"
    );
}

#[cfg(feature = "cli")]
#[test]
fn format_session_list_row_includes_total_turns() {
    let mut record = sample_record("sess-1");
    record.total_turns = 42;
    let wf = std::collections::HashMap::new();

    let row = stencila_agents::cli::format_session_list_row(&record, &wf);
    assert_eq!(row.total_turns, "42");
}

#[cfg(feature = "cli")]
#[test]
fn format_session_list_row_includes_updated_at() {
    let mut record = sample_record("sess-1");
    record.updated_at = "2025-07-01T12:34:56Z".to_string();
    let wf = std::collections::HashMap::new();

    let row = stencila_agents::cli::format_session_list_row(&record, &wf);
    assert!(
        !row.updated_at.is_empty(),
        "updated_at column should not be empty"
    );
}

// ===========================================================================
// AC-2: format_session_detail produces expected output
// ===========================================================================

/// `format_session_detail` should produce a multi-line string with key
/// session metadata.
#[cfg(feature = "cli")]
#[test]
fn format_session_detail_contains_session_id() {
    let record = sample_record("sess-detail-1");

    let detail = stencila_agents::cli::format_session_detail(&record, &[]);
    assert!(
        detail.contains("sess-detail-1"),
        "detail should contain the full session ID"
    );
}

#[cfg(feature = "cli")]
#[test]
fn format_session_detail_contains_agent_name() {
    let mut record = sample_record("sess-detail-2");
    record.agent_name = "data-analyst".to_string();

    let detail = stencila_agents::cli::format_session_detail(&record, &[]);
    assert!(
        detail.contains("data-analyst"),
        "detail should contain the agent name"
    );
}

#[cfg(feature = "cli")]
#[test]
fn format_session_detail_contains_provider_and_model() {
    let mut record = sample_record("sess-detail-3");
    record.provider_name = "openai".to_string();
    record.model_name = "gpt-4o".to_string();

    let detail = stencila_agents::cli::format_session_detail(&record, &[]);
    assert!(
        detail.contains("openai"),
        "detail should contain provider name"
    );
    assert!(
        detail.contains("gpt-4o"),
        "detail should contain model name"
    );
}

#[cfg(feature = "cli")]
#[test]
fn format_session_detail_contains_state_and_turns() {
    let mut record = sample_record("sess-detail-4");
    record.state = SessionState::Idle;
    record.total_turns = 7;

    let detail = stencila_agents::cli::format_session_detail(&record, &[]);
    assert!(
        detail.contains("7"),
        "detail should contain the total turns count"
    );
}

#[cfg(feature = "cli")]
#[test]
fn format_session_detail_includes_turns_when_provided() {
    let record = sample_record("sess-detail-5");
    let turns = vec![
        stencila_agents::types::Turn::user("Hello"),
        stencila_agents::types::Turn::assistant("Hi there!"),
    ];

    let detail = stencila_agents::cli::format_session_detail(&record, &turns);
    assert!(
        detail.contains("Hello"),
        "detail should contain user turn content"
    );
    assert!(
        detail.contains("Hi there!"),
        "detail should contain assistant turn content"
    );
}

#[cfg(feature = "cli")]
#[test]
fn format_session_detail_shows_resumability() {
    let mut record = sample_record("sess-detail-6");
    record.resumability = Resumability::None;

    let detail = stencila_agents::cli::format_session_detail(&record, &[]);
    // Should indicate the session is not resumable in some way
    assert!(
        detail.to_lowercase().contains("none")
            || detail.to_lowercase().contains("no")
            || detail.to_lowercase().contains("non"),
        "detail should indicate non-resumable status"
    );
}

// ===========================================================================
// AC-3: --resumable filtering works on the store layer
// ===========================================================================

#[test]
fn resumable_filter_returns_only_full_resumability_sessions() {
    let db = setup_db();
    let store = make_store(&db);

    let mut resumable = sample_record("sess-resumable");
    resumable.resumability = Resumability::Full;
    resumable.updated_at = "2025-07-01T00:01:00Z".to_string();
    store.insert_session(&resumable).expect("insert resumable");

    let mut not_resumable = sample_record("sess-not-resumable");
    not_resumable.resumability = Resumability::None;
    not_resumable.updated_at = "2025-07-01T00:02:00Z".to_string();
    store
        .insert_session(&not_resumable)
        .expect("insert not-resumable");

    let filter = ListSessionsFilter {
        resumable: Some(true),
        ..Default::default()
    };

    let results = store.list_sessions(&filter).expect("list_sessions");

    assert_eq!(results.len(), 1, "should only return resumable sessions");
    assert_eq!(results[0].session_id, "sess-resumable");
    assert_eq!(results[0].resumability, Resumability::Full);
}

#[test]
fn no_resumable_filter_returns_all_sessions() {
    let db = setup_db();
    let store = make_store(&db);

    let mut r1 = sample_record("sess-all-1");
    r1.resumability = Resumability::Full;
    r1.updated_at = "2025-07-01T00:01:00Z".to_string();
    store.insert_session(&r1).expect("insert r1");

    let mut r2 = sample_record("sess-all-2");
    r2.resumability = Resumability::None;
    r2.updated_at = "2025-07-01T00:02:00Z".to_string();
    store.insert_session(&r2).expect("insert r2");

    let results = store
        .list_sessions(&ListSessionsFilter::default())
        .expect("list_sessions");

    assert_eq!(
        results.len(),
        2,
        "should return all sessions without filter"
    );
}

// ===========================================================================
// AC-4: filter_sessions_for_list applies --resumable flag
// ===========================================================================

/// The `filter_sessions_for_list` function converts CLI flags into
/// a `ListSessionsFilter` for querying the store.
#[cfg(feature = "cli")]
#[test]
fn filter_sessions_for_list_with_resumable_flag() {
    let filter = stencila_agents::cli::filter_sessions_for_list(true);
    assert_eq!(
        filter.resumable,
        Some(true),
        "--resumable flag should set resumable filter to Some(true)"
    );
}

#[cfg(feature = "cli")]
#[test]
fn filter_sessions_for_list_without_resumable_flag() {
    let filter = stencila_agents::cli::filter_sessions_for_list(false);
    assert_eq!(
        filter.resumable, None,
        "no --resumable flag should leave resumable filter as None"
    );
}

// ===========================================================================
// AC-5: `Sessions` subcommand is wired into the agents CLI parser
// ===========================================================================

/// The `Command` enum in cli.rs should have a `Sessions` variant.
/// This test verifies it by checking that `stencila agents sessions`
/// parses without error (via clap's `try_parse_from`).
#[cfg(feature = "cli")]
#[test]
fn cli_parser_accepts_sessions_list_subcommand() {
    use clap::Parser;

    // This should parse successfully once the `Sessions` variant is added
    let result = stencila_agents::cli::Cli::try_parse_from(["agents", "sessions", "list"]);
    assert!(
        result.is_ok(),
        "CLI should accept `agents sessions list`: {result:?}"
    );
}

#[cfg(feature = "cli")]
#[test]
fn cli_parser_accepts_sessions_show_subcommand() {
    use clap::Parser;

    let result = stencila_agents::cli::Cli::try_parse_from([
        "agents",
        "sessions",
        "show",
        "some-session-id",
    ]);
    assert!(
        result.is_ok(),
        "CLI should accept `agents sessions show <id>`: {result:?}"
    );
}

#[cfg(feature = "cli")]
#[test]
fn cli_parser_accepts_sessions_list_with_resumable_flag() {
    use clap::Parser;

    let result =
        stencila_agents::cli::Cli::try_parse_from(["agents", "sessions", "list", "--resumable"]);
    assert!(
        result.is_ok(),
        "CLI should accept `agents sessions list --resumable`: {result:?}"
    );
}

// ===========================================================================
// AC-6: open_workspace_db_for_agents helper
// ===========================================================================

/// `open_workspace_db_for_agents` should open a WorkspaceDb at the given
/// workspace root, apply agent migrations, and return a usable store.
#[cfg(feature = "cli")]
#[test]
fn open_workspace_db_for_agents_creates_store() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let stencila_dir = tmp.path().join(".stencila");
    std::fs::create_dir_all(&stencila_dir).expect("create .stencila dir");

    let (store, _db) = stencila_agents::cli::open_workspace_db_for_agents(tmp.path())
        .expect("open_workspace_db_for_agents");

    // Should be able to insert and read back a session
    store
        .insert_session(&sample_record("sess-wiring-1"))
        .expect("insert via new store");
    let got = store
        .get_session("sess-wiring-1")
        .expect("get")
        .expect("exists");
    assert_eq!(got.session_id, "sess-wiring-1");
}

#[cfg(feature = "cli")]
#[test]
fn open_workspace_db_for_agents_applies_migrations() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let stencila_dir = tmp.path().join(".stencila");
    std::fs::create_dir_all(&stencila_dir).expect("create .stencila dir");

    let (_store, db) = stencila_agents::cli::open_workspace_db_for_agents(tmp.path())
        .expect("open_workspace_db_for_agents");

    // Check that the agent_sessions table exists via the DB connection
    let conn = db.connection().lock().expect("lock");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='agent_sessions'",
            [],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(count, 1, "agent_sessions table should exist after open");
}
