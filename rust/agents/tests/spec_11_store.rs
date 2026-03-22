//! Agent Session Store tests
//!
//! Covers the `002_agent_sessions.sql` migration, `AgentSessionStore`,
//! `insert_session`, `get_session`, `checkpoint_turns`, `get_turns`,
//! `list_sessions` (with filters), `update_session`, and the single-writer
//! lease mechanism (`acquire_lease`, `renew_lease`, `release_lease`).
//! All tests use in-memory SQLite.

#![allow(clippy::result_large_err)]

use std::sync::{Arc, Mutex};

use stencila_agents::migrations::AGENT_MIGRATIONS;
use stencila_agents::store::{
    AgentSessionStore, ListSessionsFilter, PersistencePolicy, Resumability, SessionOwner,
    SessionPersistence, SessionRecord, UpdateSession,
};
use stencila_agents::types::{SessionState, Turn};

// ===========================================================================
// Helpers
// ===========================================================================

/// Create an in-memory SQLite database with all agent migrations applied.
fn setup_db() -> Arc<Mutex<stencila_db::rusqlite::Connection>> {
    let conn = stencila_db::rusqlite::Connection::open_in_memory().expect("open in-memory SQLite");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("enable FK");
    for m in AGENT_MIGRATIONS {
        conn.execute_batch(m.sql).expect("apply migration");
    }
    Arc::new(Mutex::new(conn))
}

/// Build a store backed by the given in-memory connection.
fn make_store(conn: &Arc<Mutex<stencila_db::rusqlite::Connection>>) -> AgentSessionStore {
    AgentSessionStore::new(conn.clone())
}

/// Build a minimal `SessionRecord` with sensible defaults.
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
// AC-1: Migration creates tables and indexes
// ===========================================================================

#[test]
fn migration_creates_agent_sessions_table() {
    let db = setup_db();
    let conn = db.lock().expect("lock");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='agent_sessions'",
            [],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(
        count, 1,
        "agent_sessions table should exist after migration"
    );
}

#[test]
fn migration_creates_agent_session_turns_table() {
    let db = setup_db();
    let conn = db.lock().expect("lock");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='agent_session_turns'",
            [],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(
        count, 1,
        "agent_session_turns table should exist after migration"
    );
}

#[test]
fn migration_agent_sessions_has_required_columns() {
    let db = setup_db();
    let conn = db.lock().expect("lock");

    let expected_columns = [
        "session_id",
        "backend_kind",
        "agent_name",
        "provider_name",
        "model_name",
        "state",
        "total_turns",
        "resumability",
        "created_at",
        "updated_at",
        "workflow_run_id",
        "workflow_thread_id",
        "workflow_node_id",
        "provider_resume_state",
        "config_snapshot",
        "system_prompt",
        "lease_holder",
        "lease_expires_at",
    ];

    let mut stmt = conn
        .prepare("PRAGMA table_info(agent_sessions)")
        .expect("prepare");
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .expect("query")
        .map(|r| r.expect("row"))
        .collect();

    for col in &expected_columns {
        assert!(
            columns.contains(&col.to_string()),
            "agent_sessions missing column: {col}. Found: {columns:?}"
        );
    }
}

#[test]
fn migration_agent_session_turns_has_required_columns() {
    let db = setup_db();
    let conn = db.lock().expect("lock");

    let expected_columns = ["session_id", "turn_index", "turn_json"];

    let mut stmt = conn
        .prepare("PRAGMA table_info(agent_session_turns)")
        .expect("prepare");
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .expect("query")
        .map(|r| r.expect("row"))
        .collect();

    for col in &expected_columns {
        assert!(
            columns.contains(&col.to_string()),
            "agent_session_turns missing column: {col}. Found: {columns:?}"
        );
    }
}

#[test]
fn migration_creates_indexes_on_agent_sessions() {
    let db = setup_db();
    let conn = db.lock().expect("lock");

    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='agent_sessions' AND name NOT LIKE 'sqlite_%'",
        )
        .expect("prepare");
    let indexes: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .expect("query")
        .map(|r| r.expect("row"))
        .collect();

    assert!(
        !indexes.is_empty(),
        "agent_sessions should have at least one index"
    );
}

// ===========================================================================
// AC-2: AGENT_MIGRATIONS includes version 2
// ===========================================================================

#[test]
fn agent_migrations_includes_version_two() {
    let has_v2 = AGENT_MIGRATIONS.iter().any(|m| m.version == 2);
    assert!(has_v2, "AGENT_MIGRATIONS must include version 2");
}

#[test]
fn agent_migrations_are_ordered() {
    let versions: Vec<i32> = AGENT_MIGRATIONS.iter().map(|m| m.version).collect();
    for window in versions.windows(2) {
        assert!(
            window[0] < window[1],
            "migrations must be ordered: found {} before {}",
            window[0],
            window[1]
        );
    }
}

// ===========================================================================
// AC-3: insert_session + get_session round-trip
// ===========================================================================

#[test]
fn insert_and_get_session_roundtrip_all_fields() {
    let db = setup_db();
    let store = make_store(&db);

    let record = SessionRecord {
        session_id: "sess_rt_1".to_string(),
        backend_kind: "api".to_string(),
        agent_name: "general".to_string(),
        provider_name: "anthropic".to_string(),
        model_name: "claude-sonnet-4-20250514".to_string(),
        state: SessionState::Idle,
        total_turns: 5,
        resumability: Resumability::Full,
        created_at: "2025-07-01T00:00:00Z".to_string(),
        updated_at: "2025-07-01T00:01:00Z".to_string(),
        workflow_run_id: Some("wf_run_1".to_string()),
        workflow_thread_id: Some("wf_thread_1".to_string()),
        workflow_node_id: Some("wf_node_1".to_string()),
        provider_resume_state: Some(r#"{"key":"value"}"#.to_string()),
        config_snapshot: Some(r#"{"max_turns":100}"#.to_string()),
        system_prompt: Some("You are helpful.".to_string()),
        lease_holder: Some("host_1".to_string()),
        lease_expires_at: Some("2025-07-01T00:10:00Z".to_string()),
    };

    store.insert_session(&record).expect("insert_session");

    let got = store
        .get_session("sess_rt_1")
        .expect("get_session")
        .expect("session should exist");

    assert_eq!(got.session_id, record.session_id);
    assert_eq!(got.backend_kind, record.backend_kind);
    assert_eq!(got.agent_name, record.agent_name);
    assert_eq!(got.provider_name, record.provider_name);
    assert_eq!(got.model_name, record.model_name);
    assert_eq!(got.state, record.state);
    assert_eq!(got.total_turns, record.total_turns);
    assert_eq!(got.resumability, record.resumability);
    assert_eq!(got.created_at, record.created_at);
    assert_eq!(got.updated_at, record.updated_at);
    assert_eq!(got.workflow_run_id, record.workflow_run_id);
    assert_eq!(got.workflow_thread_id, record.workflow_thread_id);
    assert_eq!(got.workflow_node_id, record.workflow_node_id);
    assert_eq!(got.provider_resume_state, record.provider_resume_state);
    assert_eq!(got.config_snapshot, record.config_snapshot);
    assert_eq!(got.system_prompt, record.system_prompt);
    assert_eq!(got.lease_holder, record.lease_holder);
    assert_eq!(got.lease_expires_at, record.lease_expires_at);
}

#[test]
fn insert_and_get_session_with_none_optional_fields() {
    let db = setup_db();
    let store = make_store(&db);

    let record = sample_record("sess_none_1");
    store.insert_session(&record).expect("insert_session");

    let got = store
        .get_session("sess_none_1")
        .expect("get_session")
        .expect("session should exist");

    assert_eq!(got.workflow_run_id, None);
    assert_eq!(got.workflow_thread_id, None);
    assert_eq!(got.workflow_node_id, None);
    assert_eq!(got.provider_resume_state, None);
    assert_eq!(got.config_snapshot, None);
    assert_eq!(got.system_prompt, None);
    assert_eq!(got.lease_holder, None);
    assert_eq!(got.lease_expires_at, None);
}

#[test]
fn get_session_returns_none_for_missing_id() {
    let db = setup_db();
    let store = make_store(&db);

    let result = store
        .get_session("nonexistent")
        .expect("get_session should not error");
    assert!(result.is_none(), "missing session should return None");
}

// ===========================================================================
// AC-4: checkpoint_turns + get_turns round-trip
// ===========================================================================

#[test]
fn checkpoint_and_get_turns_roundtrip() {
    let db = setup_db();
    let store = make_store(&db);

    let record = sample_record("sess_turns_1");
    store.insert_session(&record).expect("insert_session");

    let turns = vec![
        Turn::user("Hello, world!"),
        Turn::assistant("Hi there!"),
        Turn::user("How are you?"),
    ];

    store
        .checkpoint_turns("sess_turns_1", &turns)
        .expect("checkpoint_turns");

    let got = store.get_turns("sess_turns_1").expect("get_turns");

    assert_eq!(got.len(), turns.len(), "turn count mismatch");
    for (i, (expected, actual)) in turns.iter().zip(got.iter()).enumerate() {
        assert_eq!(expected, actual, "turn {i} mismatch");
    }
}

#[test]
fn get_turns_returns_empty_for_no_checkpoints() {
    let db = setup_db();
    let store = make_store(&db);

    let record = sample_record("sess_empty_turns");
    store.insert_session(&record).expect("insert_session");

    let turns = store.get_turns("sess_empty_turns").expect("get_turns");
    assert!(turns.is_empty(), "no checkpointed turns should be empty");
}

#[test]
fn checkpoint_turns_preserves_turn_order() {
    let db = setup_db();
    let store = make_store(&db);

    let record = sample_record("sess_order_1");
    store.insert_session(&record).expect("insert_session");

    let turns: Vec<Turn> = (0..10)
        .map(|i| Turn::user(format!("message {i}")))
        .collect();

    store
        .checkpoint_turns("sess_order_1", &turns)
        .expect("checkpoint_turns");

    let got = store.get_turns("sess_order_1").expect("get_turns");

    for (i, turn) in got.iter().enumerate() {
        if let Turn::User { content, .. } = turn {
            assert_eq!(
                *content,
                format!("message {i}"),
                "turn order mismatch at {i}"
            );
        } else {
            panic!("expected User turn at index {i}, got {turn:?}");
        }
    }
}

#[test]
fn checkpoint_turns_stores_complex_turn_types() {
    let db = setup_db();
    let store = make_store(&db);

    let record = sample_record("sess_complex_turns");
    store.insert_session(&record).expect("insert_session");

    let turns = vec![
        Turn::system("You are helpful."),
        Turn::user("Read file."),
        Turn::assistant("Let me read that."),
        Turn::steering("try a different approach"),
    ];

    store
        .checkpoint_turns("sess_complex_turns", &turns)
        .expect("checkpoint_turns");

    let got = store.get_turns("sess_complex_turns").expect("get_turns");
    assert_eq!(got.len(), 4);
    assert_eq!(got, turns);
}

// ===========================================================================
// AC-5: Second checkpoint_turns replaces the first (snapshot semantics)
// ===========================================================================

#[test]
fn checkpoint_turns_replaces_previous_snapshot() {
    let db = setup_db();
    let store = make_store(&db);

    let record = sample_record("sess_replace_1");
    store.insert_session(&record).expect("insert_session");

    // First checkpoint
    let turns_v1 = vec![Turn::user("first"), Turn::assistant("response 1")];
    store
        .checkpoint_turns("sess_replace_1", &turns_v1)
        .expect("first checkpoint");

    // Second checkpoint replaces first
    let turns_v2 = vec![
        Turn::user("first"),
        Turn::assistant("response 1"),
        Turn::user("second"),
        Turn::assistant("response 2"),
    ];
    store
        .checkpoint_turns("sess_replace_1", &turns_v2)
        .expect("second checkpoint");

    let got = store.get_turns("sess_replace_1").expect("get_turns");
    assert_eq!(
        got.len(),
        turns_v2.len(),
        "should have turns from second checkpoint, not accumulated"
    );
    assert_eq!(got, turns_v2);
}

#[test]
fn checkpoint_turns_replace_with_fewer_turns() {
    let db = setup_db();
    let store = make_store(&db);

    let record = sample_record("sess_fewer_1");
    store.insert_session(&record).expect("insert_session");

    // Checkpoint with 5 turns
    let turns_v1: Vec<Turn> = (0..5).map(|i| Turn::user(format!("msg {i}"))).collect();
    store
        .checkpoint_turns("sess_fewer_1", &turns_v1)
        .expect("first checkpoint");

    // Replace with 2 turns — old rows must be deleted
    let turns_v2 = vec![Turn::user("only one"), Turn::assistant("only two")];
    store
        .checkpoint_turns("sess_fewer_1", &turns_v2)
        .expect("second checkpoint");

    let got = store.get_turns("sess_fewer_1").expect("get_turns");
    assert_eq!(got.len(), 2, "old turns should be deleted");
    assert_eq!(got, turns_v2);
}

#[test]
fn checkpoint_turns_replace_with_empty() {
    let db = setup_db();
    let store = make_store(&db);

    let record = sample_record("sess_empty_replace");
    store.insert_session(&record).expect("insert_session");

    let turns_v1 = vec![Turn::user("hello")];
    store
        .checkpoint_turns("sess_empty_replace", &turns_v1)
        .expect("first checkpoint");

    // Replace with empty
    store
        .checkpoint_turns("sess_empty_replace", &[])
        .expect("checkpoint with empty");

    let got = store.get_turns("sess_empty_replace").expect("get_turns");
    assert!(
        got.is_empty(),
        "turns should be empty after empty checkpoint"
    );
}

// ===========================================================================
// Store types exist and have expected variants/fields
// ===========================================================================

#[test]
fn session_persistence_type_exists() {
    let _sp = SessionPersistence::Persistent;
}

#[test]
fn persistence_policy_type_exists() {
    let _pp = PersistencePolicy::default();
}

#[test]
fn session_owner_type_exists() {
    let _owner = SessionOwner {
        user_id: Some("user_1".to_string()),
        workspace_id: Some("ws_1".to_string()),
    };
    assert_eq!(_owner.user_id.as_deref(), Some("user_1"));
}

#[test]
fn resumability_type_exists_with_variants() {
    let _full = Resumability::Full;
    let _none = Resumability::None;
}

#[test]
fn session_record_debug_and_clone() {
    let record = sample_record("sess_debug");
    let cloned = record.clone();
    assert_eq!(
        format!("{:?}", record),
        format!("{:?}", cloned),
        "SessionRecord should be Debug + Clone"
    );
}

// ===========================================================================
// AC-S3-1: list_sessions with no filter returns all sessions ordered by
//          updated_at DESC, session_id DESC
// ===========================================================================

#[test]
fn list_sessions_no_filter_returns_all() {
    let db = setup_db();
    let store = make_store(&db);

    store
        .insert_session(&sample_record("sess_a"))
        .expect("insert a");
    store
        .insert_session(&sample_record("sess_b"))
        .expect("insert b");
    store
        .insert_session(&sample_record("sess_c"))
        .expect("insert c");

    let results = store
        .list_sessions(&ListSessionsFilter::default())
        .expect("list_sessions");

    assert_eq!(results.len(), 3, "should return all 3 sessions");
}

#[test]
fn list_sessions_no_filter_ordered_by_updated_at_desc_then_session_id_desc() {
    let db = setup_db();
    let store = make_store(&db);

    // Insert sessions with different updated_at values
    let mut r1 = sample_record("sess_oldest");
    r1.updated_at = "2025-07-01T00:00:00Z".to_string();
    store.insert_session(&r1).expect("insert oldest");

    let mut r2 = sample_record("sess_newest");
    r2.updated_at = "2025-07-01T02:00:00Z".to_string();
    store.insert_session(&r2).expect("insert newest");

    let mut r3 = sample_record("sess_middle");
    r3.updated_at = "2025-07-01T01:00:00Z".to_string();
    store.insert_session(&r3).expect("insert middle");

    let results = store
        .list_sessions(&ListSessionsFilter::default())
        .expect("list_sessions");

    assert_eq!(results.len(), 3);
    assert_eq!(
        results[0].session_id, "sess_newest",
        "first should be newest"
    );
    assert_eq!(
        results[1].session_id, "sess_middle",
        "second should be middle"
    );
    assert_eq!(
        results[2].session_id, "sess_oldest",
        "third should be oldest"
    );
}

#[test]
fn list_sessions_no_filter_tiebreak_by_session_id_desc() {
    let db = setup_db();
    let store = make_store(&db);

    // Same updated_at — tiebreak by session_id DESC
    let same_time = "2025-07-01T00:00:00Z";

    let mut ra = sample_record("sess_aaa");
    ra.updated_at = same_time.to_string();
    store.insert_session(&ra).expect("insert aaa");

    let mut rz = sample_record("sess_zzz");
    rz.updated_at = same_time.to_string();
    store.insert_session(&rz).expect("insert zzz");

    let mut rm = sample_record("sess_mmm");
    rm.updated_at = same_time.to_string();
    store.insert_session(&rm).expect("insert mmm");

    let results = store
        .list_sessions(&ListSessionsFilter::default())
        .expect("list_sessions");

    assert_eq!(results.len(), 3);
    assert_eq!(
        results[0].session_id, "sess_zzz",
        "first by session_id DESC"
    );
    assert_eq!(results[1].session_id, "sess_mmm");
    assert_eq!(results[2].session_id, "sess_aaa", "last by session_id DESC");
}

#[test]
fn list_sessions_empty_table_returns_empty() {
    let db = setup_db();
    let store = make_store(&db);

    let results = store
        .list_sessions(&ListSessionsFilter::default())
        .expect("list_sessions");

    assert!(results.is_empty(), "no sessions inserted, should be empty");
}

// ===========================================================================
// AC-S3-2: list_sessions with resumable filter returns only resumable sessions
// ===========================================================================

#[test]
fn list_sessions_resumable_filter_returns_only_resumable() {
    let db = setup_db();
    let store = make_store(&db);

    let mut resumable = sample_record("sess_resumable");
    resumable.resumability = Resumability::Full;
    store.insert_session(&resumable).expect("insert resumable");

    let mut non_resumable = sample_record("sess_non_resumable");
    non_resumable.resumability = Resumability::None;
    store
        .insert_session(&non_resumable)
        .expect("insert non-resumable");

    let filter = ListSessionsFilter {
        resumable: Some(true),
        ..Default::default()
    };

    let results = store.list_sessions(&filter).expect("list_sessions");

    assert_eq!(
        results.len(),
        1,
        "only the resumable session should be returned"
    );
    assert_eq!(results[0].session_id, "sess_resumable");
    assert_eq!(results[0].resumability, Resumability::Full);
}

#[test]
fn list_sessions_resumable_filter_with_no_matches() {
    let db = setup_db();
    let store = make_store(&db);

    let mut non_resumable = sample_record("sess_nr_only");
    non_resumable.resumability = Resumability::None;
    store.insert_session(&non_resumable).expect("insert");

    let filter = ListSessionsFilter {
        resumable: Some(true),
        ..Default::default()
    };

    let results = store.list_sessions(&filter).expect("list_sessions");
    assert!(results.is_empty(), "no resumable sessions should match");
}

// ===========================================================================
// AC-S3-3: list_sessions with workflow ownership filter
// ===========================================================================

#[test]
fn list_sessions_workflow_filter_matches_run_and_thread() {
    let db = setup_db();
    let store = make_store(&db);

    let mut wf_session = sample_record("sess_wf_1");
    wf_session.workflow_run_id = Some("run_abc".to_string());
    wf_session.workflow_thread_id = Some("thread_xyz".to_string());
    store
        .insert_session(&wf_session)
        .expect("insert wf session");

    let mut other_session = sample_record("sess_other");
    other_session.workflow_run_id = Some("run_other".to_string());
    other_session.workflow_thread_id = Some("thread_other".to_string());
    store.insert_session(&other_session).expect("insert other");

    let plain_session = sample_record("sess_plain");
    // No workflow fields
    store.insert_session(&plain_session).expect("insert plain");

    let filter = ListSessionsFilter {
        workflow_run_id: Some("run_abc".to_string()),
        workflow_thread_id: Some("thread_xyz".to_string()),
        ..Default::default()
    };

    let results = store.list_sessions(&filter).expect("list_sessions");

    assert_eq!(results.len(), 1, "only the matching workflow session");
    assert_eq!(results[0].session_id, "sess_wf_1");
}

#[test]
fn list_sessions_workflow_filter_run_id_only() {
    let db = setup_db();
    let store = make_store(&db);

    let mut s1 = sample_record("sess_run_match");
    s1.workflow_run_id = Some("run_123".to_string());
    s1.workflow_thread_id = Some("thread_a".to_string());
    store.insert_session(&s1).expect("insert s1");

    let mut s2 = sample_record("sess_run_match_2");
    s2.workflow_run_id = Some("run_123".to_string());
    s2.workflow_thread_id = Some("thread_b".to_string());
    store.insert_session(&s2).expect("insert s2");

    let mut s3 = sample_record("sess_no_match");
    s3.workflow_run_id = Some("run_other".to_string());
    store.insert_session(&s3).expect("insert s3");

    let filter = ListSessionsFilter {
        workflow_run_id: Some("run_123".to_string()),
        ..Default::default()
    };

    let results = store.list_sessions(&filter).expect("list_sessions");

    assert_eq!(results.len(), 2, "both sessions with run_123 should match");
    let ids: Vec<&str> = results.iter().map(|r| r.session_id.as_str()).collect();
    assert!(ids.contains(&"sess_run_match"));
    assert!(ids.contains(&"sess_run_match_2"));
}

#[test]
fn list_sessions_combined_resumable_and_workflow_filter() {
    let db = setup_db();
    let store = make_store(&db);

    // Session that matches both filters
    let mut match_both = sample_record("sess_both");
    match_both.resumability = Resumability::Full;
    match_both.workflow_run_id = Some("run_x".to_string());
    store
        .insert_session(&match_both)
        .expect("insert match_both");

    // Resumable but wrong workflow
    let mut resumable_wrong_wf = sample_record("sess_res_wrong");
    resumable_wrong_wf.resumability = Resumability::Full;
    resumable_wrong_wf.workflow_run_id = Some("run_other".to_string());
    store.insert_session(&resumable_wrong_wf).expect("insert");

    // Right workflow but not resumable
    let mut not_resumable = sample_record("sess_nr_right");
    not_resumable.resumability = Resumability::None;
    not_resumable.workflow_run_id = Some("run_x".to_string());
    store.insert_session(&not_resumable).expect("insert");

    let filter = ListSessionsFilter {
        resumable: Some(true),
        workflow_run_id: Some("run_x".to_string()),
        ..Default::default()
    };

    let results = store.list_sessions(&filter).expect("list_sessions");

    assert_eq!(results.len(), 1, "only the session matching both filters");
    assert_eq!(results[0].session_id, "sess_both");
}

// ===========================================================================
// AC-S3-4: update_session
// ===========================================================================

#[test]
fn update_session_changes_state_and_total_turns() {
    let db = setup_db();
    let store = make_store(&db);

    store
        .insert_session(&sample_record("sess_upd_1"))
        .expect("insert");

    let update = UpdateSession {
        state: SessionState::Processing,
        total_turns: 10,
        updated_at: "2025-07-01T01:00:00Z".to_string(),
        resumability: None,
    };

    store
        .update_session("sess_upd_1", &update)
        .expect("update_session");

    let got = store
        .get_session("sess_upd_1")
        .expect("get")
        .expect("exists");

    assert_eq!(got.state, SessionState::Processing);
    assert_eq!(got.total_turns, 10);
    assert_eq!(got.updated_at, "2025-07-01T01:00:00Z");
}

#[test]
fn update_session_changes_resumability_when_provided() {
    let db = setup_db();
    let store = make_store(&db);

    let mut record = sample_record("sess_upd_res");
    record.resumability = Resumability::Full;
    store.insert_session(&record).expect("insert");

    let update = UpdateSession {
        state: SessionState::Closed,
        total_turns: 20,
        updated_at: "2025-07-01T02:00:00Z".to_string(),
        resumability: Some(Resumability::None),
    };

    store
        .update_session("sess_upd_res", &update)
        .expect("update_session");

    let got = store
        .get_session("sess_upd_res")
        .expect("get")
        .expect("exists");

    assert_eq!(got.resumability, Resumability::None);
    assert_eq!(got.state, SessionState::Closed);
}

#[test]
fn update_session_leaves_resumability_unchanged_when_none() {
    let db = setup_db();
    let store = make_store(&db);

    let mut record = sample_record("sess_upd_keep");
    record.resumability = Resumability::Full;
    store.insert_session(&record).expect("insert");

    let update = UpdateSession {
        state: SessionState::AwaitingInput,
        total_turns: 3,
        updated_at: "2025-07-01T00:05:00Z".to_string(),
        resumability: None, // do not change
    };

    store
        .update_session("sess_upd_keep", &update)
        .expect("update_session");

    let got = store
        .get_session("sess_upd_keep")
        .expect("get")
        .expect("exists");

    assert_eq!(
        got.resumability,
        Resumability::Full,
        "resumability should be unchanged"
    );
    assert_eq!(got.state, SessionState::AwaitingInput);
}

#[test]
fn update_session_preserves_other_fields() {
    let db = setup_db();
    let store = make_store(&db);

    let record = SessionRecord {
        session_id: "sess_upd_preserve".to_string(),
        backend_kind: "api".to_string(),
        agent_name: "general".to_string(),
        provider_name: "anthropic".to_string(),
        model_name: "claude-sonnet-4-20250514".to_string(),
        state: SessionState::Idle,
        total_turns: 0,
        resumability: Resumability::Full,
        created_at: "2025-07-01T00:00:00Z".to_string(),
        updated_at: "2025-07-01T00:00:00Z".to_string(),
        workflow_run_id: Some("wf_run".to_string()),
        workflow_thread_id: Some("wf_thread".to_string()),
        workflow_node_id: Some("wf_node".to_string()),
        provider_resume_state: Some("resume_state".to_string()),
        config_snapshot: Some("config".to_string()),
        system_prompt: Some("prompt".to_string()),
        lease_holder: Some("host".to_string()),
        lease_expires_at: Some("2025-07-01T01:00:00Z".to_string()),
    };
    store.insert_session(&record).expect("insert");

    let update = UpdateSession {
        state: SessionState::Processing,
        total_turns: 42,
        updated_at: "2025-07-01T03:00:00Z".to_string(),
        resumability: None,
    };

    store
        .update_session("sess_upd_preserve", &update)
        .expect("update_session");

    let got = store
        .get_session("sess_upd_preserve")
        .expect("get")
        .expect("exists");

    // Updated fields
    assert_eq!(got.state, SessionState::Processing);
    assert_eq!(got.total_turns, 42);
    assert_eq!(got.updated_at, "2025-07-01T03:00:00Z");

    // Preserved fields
    assert_eq!(got.backend_kind, "api");
    assert_eq!(got.agent_name, "general");
    assert_eq!(got.provider_name, "anthropic");
    assert_eq!(got.model_name, "claude-sonnet-4-20250514");
    assert_eq!(got.created_at, "2025-07-01T00:00:00Z");
    assert_eq!(got.workflow_run_id.as_deref(), Some("wf_run"));
    assert_eq!(got.workflow_thread_id.as_deref(), Some("wf_thread"));
    assert_eq!(got.workflow_node_id.as_deref(), Some("wf_node"));
    assert_eq!(got.provider_resume_state.as_deref(), Some("resume_state"));
    assert_eq!(got.config_snapshot.as_deref(), Some("config"));
    assert_eq!(got.system_prompt.as_deref(), Some("prompt"));
    assert_eq!(got.lease_holder.as_deref(), Some("host"));
    assert_eq!(
        got.lease_expires_at.as_deref(),
        Some("2025-07-01T01:00:00Z")
    );
}

#[test]
fn update_session_reflects_in_list_sessions_order() {
    let db = setup_db();
    let store = make_store(&db);

    // Insert two sessions with oldest having the earlier updated_at
    let mut old = sample_record("sess_was_old");
    old.updated_at = "2025-07-01T00:00:00Z".to_string();
    store.insert_session(&old).expect("insert old");

    let mut newer = sample_record("sess_was_newer");
    newer.updated_at = "2025-07-01T01:00:00Z".to_string();
    store.insert_session(&newer).expect("insert newer");

    // Update the old session to have the latest updated_at
    let update = UpdateSession {
        state: SessionState::Processing,
        total_turns: 5,
        updated_at: "2025-07-01T02:00:00Z".to_string(),
        resumability: None,
    };
    store
        .update_session("sess_was_old", &update)
        .expect("update");

    let results = store
        .list_sessions(&ListSessionsFilter::default())
        .expect("list_sessions");

    assert_eq!(results.len(), 2);
    assert_eq!(
        results[0].session_id, "sess_was_old",
        "updated session should now be first (newest updated_at)"
    );
    assert_eq!(results[1].session_id, "sess_was_newer");
}

// ===========================================================================
// AC-P2-1: acquire_lease succeeds when no active lease exists
// ===========================================================================

#[test]
fn acquire_lease_succeeds_when_no_lease() {
    let db = setup_db();
    let store = make_store(&db);

    store
        .insert_session(&sample_record("sess_lease_1"))
        .expect("insert");

    let acquired = store
        .acquire_lease("sess_lease_1", "holder_a", "2025-07-01T01:00:00Z")
        .expect("acquire_lease");

    assert!(acquired, "should acquire lease when none exists");

    let got = store
        .get_session("sess_lease_1")
        .expect("get")
        .expect("exists");

    assert_eq!(got.lease_holder.as_deref(), Some("holder_a"));
    assert_eq!(
        got.lease_expires_at.as_deref(),
        Some("2025-07-01T01:00:00Z")
    );
}

#[test]
fn acquire_lease_succeeds_when_same_holder_already_holds() {
    let db = setup_db();
    let store = make_store(&db);

    store
        .insert_session(&sample_record("sess_lease_same"))
        .expect("insert");

    let first = store
        .acquire_lease("sess_lease_same", "holder_a", "2025-07-01T01:00:00Z")
        .expect("first acquire");
    assert!(first);

    // Same holder re-acquires — should succeed (idempotent)
    let second = store
        .acquire_lease("sess_lease_same", "holder_a", "2025-07-01T02:00:00Z")
        .expect("second acquire by same holder");
    assert!(
        second,
        "same holder should be able to re-acquire their own active lease"
    );
}

// ===========================================================================
// AC-P2-2: acquire_lease fails when active lease held by different holder
// ===========================================================================

#[test]
fn acquire_lease_fails_when_held_by_different_holder() {
    let db = setup_db();
    let store = make_store(&db);

    store
        .insert_session(&sample_record("sess_lease_conflict"))
        .expect("insert");

    // holder_a acquires with a far-future expiry
    let first = store
        .acquire_lease("sess_lease_conflict", "holder_a", "2099-12-31T23:59:59Z")
        .expect("first acquire");
    assert!(first);

    // holder_b tries to acquire — should fail because holder_a's lease is active
    let conflict = store
        .acquire_lease("sess_lease_conflict", "holder_b", "2099-12-31T23:59:59Z")
        .expect("conflicting acquire");
    assert!(
        !conflict,
        "should not acquire when active lease held by different holder"
    );

    // Verify original holder is still set
    let got = store
        .get_session("sess_lease_conflict")
        .expect("get")
        .expect("exists");
    assert_eq!(got.lease_holder.as_deref(), Some("holder_a"));
}

// ===========================================================================
// AC-P2-3: acquire_lease succeeds when existing lease has expired
// ===========================================================================

#[test]
fn acquire_lease_succeeds_when_lease_expired() {
    let db = setup_db();
    let store = make_store(&db);

    store
        .insert_session(&sample_record("sess_lease_expired"))
        .expect("insert");

    // holder_a acquires with an already-expired timestamp
    let first = store
        .acquire_lease("sess_lease_expired", "holder_a", "2020-01-01T00:00:00Z")
        .expect("first acquire with past expiry");
    assert!(first);

    // holder_b acquires — should succeed because holder_a's lease is expired
    let second = store
        .acquire_lease("sess_lease_expired", "holder_b", "2099-12-31T23:59:59Z")
        .expect("acquire after expiry");
    assert!(
        second,
        "should acquire lease when existing lease has expired"
    );

    let got = store
        .get_session("sess_lease_expired")
        .expect("get")
        .expect("exists");
    assert_eq!(got.lease_holder.as_deref(), Some("holder_b"));
    assert_eq!(
        got.lease_expires_at.as_deref(),
        Some("2099-12-31T23:59:59Z")
    );
}

// ===========================================================================
// AC-P2-4: renew_lease succeeds for same holder
// ===========================================================================

#[test]
fn renew_lease_extends_expiry_for_same_holder() {
    let db = setup_db();
    let store = make_store(&db);

    store
        .insert_session(&sample_record("sess_renew_ok"))
        .expect("insert");

    store
        .acquire_lease("sess_renew_ok", "holder_a", "2025-07-01T01:00:00Z")
        .expect("acquire");

    let renewed = store
        .renew_lease("sess_renew_ok", "holder_a", "2025-07-01T02:00:00Z")
        .expect("renew_lease");
    assert!(renewed, "same holder should be able to renew");

    let got = store
        .get_session("sess_renew_ok")
        .expect("get")
        .expect("exists");
    assert_eq!(got.lease_holder.as_deref(), Some("holder_a"));
    assert_eq!(
        got.lease_expires_at.as_deref(),
        Some("2025-07-01T02:00:00Z"),
        "expiry should be extended"
    );
}

// ===========================================================================
// AC-P2-5: renew_lease fails for different holder
// ===========================================================================

#[test]
fn renew_lease_fails_for_different_holder() {
    let db = setup_db();
    let store = make_store(&db);

    store
        .insert_session(&sample_record("sess_renew_diff"))
        .expect("insert");

    store
        .acquire_lease("sess_renew_diff", "holder_a", "2099-12-31T23:59:59Z")
        .expect("acquire");

    let renewed = store
        .renew_lease("sess_renew_diff", "holder_b", "2099-12-31T23:59:59Z")
        .expect("renew by different holder");
    assert!(
        !renewed,
        "different holder should not be able to renew the lease"
    );

    // Original holder and expiry unchanged
    let got = store
        .get_session("sess_renew_diff")
        .expect("get")
        .expect("exists");
    assert_eq!(got.lease_holder.as_deref(), Some("holder_a"));
}

#[test]
fn renew_lease_fails_when_no_lease_exists() {
    let db = setup_db();
    let store = make_store(&db);

    store
        .insert_session(&sample_record("sess_renew_none"))
        .expect("insert");

    // No lease acquired — renew should fail
    let renewed = store
        .renew_lease("sess_renew_none", "holder_a", "2025-07-01T01:00:00Z")
        .expect("renew without existing lease");
    assert!(!renewed, "should not renew when no lease has been acquired");
}

// ===========================================================================
// AC-P2-6: release_lease clears lease; subsequent acquire succeeds
// ===========================================================================

#[test]
fn release_lease_clears_holder_and_expiry() {
    let db = setup_db();
    let store = make_store(&db);

    store
        .insert_session(&sample_record("sess_release_1"))
        .expect("insert");

    store
        .acquire_lease("sess_release_1", "holder_a", "2099-12-31T23:59:59Z")
        .expect("acquire");

    store
        .release_lease("sess_release_1")
        .expect("release_lease");

    let got = store
        .get_session("sess_release_1")
        .expect("get")
        .expect("exists");
    assert_eq!(
        got.lease_holder, None,
        "lease_holder should be cleared after release"
    );
    assert_eq!(
        got.lease_expires_at, None,
        "lease_expires_at should be cleared after release"
    );
}

#[test]
fn release_then_acquire_by_any_holder_succeeds() {
    let db = setup_db();
    let store = make_store(&db);

    store
        .insert_session(&sample_record("sess_release_acq"))
        .expect("insert");

    // holder_a acquires, then releases
    store
        .acquire_lease("sess_release_acq", "holder_a", "2099-12-31T23:59:59Z")
        .expect("acquire");
    store.release_lease("sess_release_acq").expect("release");

    // holder_b acquires after release — should succeed
    let acquired = store
        .acquire_lease("sess_release_acq", "holder_b", "2099-12-31T23:59:59Z")
        .expect("acquire after release");
    assert!(
        acquired,
        "any holder should be able to acquire after release"
    );

    let got = store
        .get_session("sess_release_acq")
        .expect("get")
        .expect("exists");
    assert_eq!(got.lease_holder.as_deref(), Some("holder_b"));
}

#[test]
fn release_lease_is_idempotent() {
    let db = setup_db();
    let store = make_store(&db);

    store
        .insert_session(&sample_record("sess_release_idem"))
        .expect("insert");

    // Release without ever acquiring — should not error
    store
        .release_lease("sess_release_idem")
        .expect("release without acquire should not error");

    // Double release after acquire
    store
        .acquire_lease("sess_release_idem", "holder_a", "2099-12-31T23:59:59Z")
        .expect("acquire");
    store
        .release_lease("sess_release_idem")
        .expect("first release");
    store
        .release_lease("sess_release_idem")
        .expect("second release should not error");

    let got = store
        .get_session("sess_release_idem")
        .expect("get")
        .expect("exists");
    assert_eq!(got.lease_holder, None);
    assert_eq!(got.lease_expires_at, None);
}
