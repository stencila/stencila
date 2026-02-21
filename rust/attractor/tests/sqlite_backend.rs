//! Tests for the SQLite context backend.
//!
//! These tests exercise the `SqliteBackend` as a `ContextBackend`
//! implementation and the run-lifecycle methods.
//!
//! Requires the `sqlite` feature.
#![cfg(feature = "sqlite")]

use indexmap::IndexMap;
use serde_json::{Value, json};
use tempfile::TempDir;

use stencila_attractor::context::Context;
use stencila_attractor::sqlite_backend::{NodeRecord, SqliteBackend};
use stencila_db::WorkspaceDb;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn temp_db() -> (TempDir, SqliteBackend) {
    let dir = TempDir::new().expect("temp dir");
    let db_path = dir.path().join("test.db");
    let workspace_db = WorkspaceDb::open(&db_path).expect("open workspace db");
    let backend = SqliteBackend::open(&workspace_db, "run-1").expect("open db");
    (dir, backend)
}

fn temp_context() -> (TempDir, Context) {
    let dir = TempDir::new().expect("temp dir");
    let db_path = dir.path().join("test.db");
    let workspace_db = WorkspaceDb::open(&db_path).expect("open workspace db");
    let ctx = Context::with_sqlite(&workspace_db, "run-1").expect("open context");

    // Insert a run row so foreign key constraints are satisfied
    if let Some(backend) = ctx.sqlite_backend() {
        backend.insert_run("test", "", "").expect("insert run");
    }

    (dir, ctx)
}

// ---------------------------------------------------------------------------
// ContextBackend trait tests
// ---------------------------------------------------------------------------

#[test]
fn set_and_get_string() {
    let (_dir, ctx) = temp_context();
    ctx.set("name", Value::String("hello".into()));
    assert_eq!(ctx.get_string("name"), "hello");
}

#[test]
fn set_and_get_number() {
    let (_dir, ctx) = temp_context();
    ctx.set("count", json!(42));
    assert_eq!(ctx.get_i64("count"), Some(42));
}

#[test]
fn get_missing_key_returns_none() {
    let (_dir, ctx) = temp_context();
    assert_eq!(ctx.get("nonexistent"), None);
    assert_eq!(ctx.get_string("nonexistent"), "");
}

#[test]
fn set_overwrites_existing() {
    let (_dir, ctx) = temp_context();
    ctx.set("key", json!("first"));
    ctx.set("key", json!("second"));
    assert_eq!(ctx.get_string("key"), "second");
}

#[test]
fn snapshot_returns_all_values() {
    let (_dir, ctx) = temp_context();
    ctx.set("a", json!(1));
    ctx.set("b", json!("two"));
    ctx.set("c", json!(true));

    let snap = ctx.snapshot();
    assert_eq!(snap.len(), 3);
    assert_eq!(snap.get("a"), Some(&json!(1)));
    assert_eq!(snap.get("b"), Some(&json!("two")));
    assert_eq!(snap.get("c"), Some(&json!(true)));
}

#[test]
fn deep_clone_is_independent() {
    let (_dir, ctx) = temp_context();
    ctx.set("x", json!(10));
    ctx.append_log("entry1");

    let cloned = ctx.deep_clone();
    cloned.set("x", json!(20));
    cloned.append_log("entry2");

    // Original unchanged (deep_clone returns in-memory backend)
    assert_eq!(ctx.get_i64("x"), Some(10));
    assert_eq!(ctx.logs().len(), 1);

    // Cloned has the updates
    assert_eq!(cloned.get_i64("x"), Some(20));
    assert_eq!(cloned.logs().len(), 2);
}

#[test]
fn apply_updates_batch() {
    let (_dir, ctx) = temp_context();
    ctx.set("existing", json!("old"));

    let mut updates = IndexMap::new();
    updates.insert("existing".to_string(), json!("new"));
    updates.insert("added".to_string(), json!(42));
    ctx.apply_updates(&updates);

    assert_eq!(ctx.get_string("existing"), "new");
    assert_eq!(ctx.get_i64("added"), Some(42));
}

#[test]
fn logs_append_and_retrieve() {
    let (_dir, ctx) = temp_context();
    ctx.append_log("first");
    ctx.append_log("second");
    ctx.append_log("third");

    let logs = ctx.logs();
    assert_eq!(logs, vec!["first", "second", "third"]);
}

// ---------------------------------------------------------------------------
// Migration tests
// ---------------------------------------------------------------------------

#[test]
fn migration_creates_tables() {
    let (_dir, backend) = temp_db();
    let conn = backend.connection().lock().expect("lock");

    // Verify key tables exist by querying sqlite_master
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN \
             ('workflow_runs', 'workflow_context', 'workflow_nodes', 'workflow_logs', \
              'workflow_edges', 'workflow_artifacts', 'workflow_interviews', \
              'workflow_node_outputs', 'workflow_definition_snapshots', \
              'workflow_run_definitions')",
            [],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(count, 10);
}

#[test]
fn migration_is_idempotent() {
    let dir = TempDir::new().expect("temp dir");
    let db_path = dir.path().join("test.db");

    // Open twice — second open should not fail
    let workspace_db = WorkspaceDb::open(&db_path).expect("open workspace db");
    let _b1 = SqliteBackend::open(&workspace_db, "run-1").expect("first open");
    let _b2 = SqliteBackend::open(&workspace_db, "run-2").expect("second open");
}

#[test]
fn migration_is_tracked() {
    let (_dir, backend) = temp_db();
    let conn = backend.connection().lock().expect("lock");
    let (domain, version, name): (String, i32, String) = conn
        .query_row(
            "SELECT domain, version, name FROM _migrations WHERE domain = 'workflows'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("query");
    assert_eq!(domain, "workflows");
    assert_eq!(version, 1);
    assert_eq!(name, "initial");
}

// ---------------------------------------------------------------------------
// Run lifecycle tests
// ---------------------------------------------------------------------------

#[test]
fn insert_and_complete_run() {
    let (_dir, backend) = temp_db();
    backend
        .insert_run("test-pipeline", "test goal", "0.1.0")
        .expect("insert run");

    let conn = backend.connection().lock().expect("lock");
    let (name, status): (String, String) = conn
        .query_row(
            "SELECT workflow_name, status FROM workflow_runs WHERE run_id = ?1",
            ("run-1",),
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("query");
    assert_eq!(name, "test-pipeline");
    assert_eq!(status, "running");
    drop(conn);

    backend
        .complete_run("completed", 1000, 0)
        .expect("complete");

    let conn = backend.connection().lock().expect("lock");
    let status: String = conn
        .query_row(
            "SELECT status FROM workflow_runs WHERE run_id = ?1",
            ("run-1",),
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(status, "completed");
}

#[test]
fn upsert_node_insert_and_update() {
    let (_dir, backend) = temp_db();
    backend.insert_run("p", "", "").expect("insert run");

    backend
        .upsert_node(&NodeRecord {
            node_id: "plan",
            status: "running",
            model: Some("gpt-5"),
            provider: Some("openai"),
            duration_ms: None,
            input_tokens: Some(100),
            output_tokens: None,
            retry_count: None,
            failure_reason: None,
        })
        .expect("insert node");

    backend
        .upsert_node(&NodeRecord {
            node_id: "plan",
            status: "completed",
            model: None,
            provider: None,
            duration_ms: Some(500),
            input_tokens: None,
            output_tokens: Some(200),
            retry_count: None,
            failure_reason: None,
        })
        .expect("update node");

    let conn = backend.connection().lock().expect("lock");
    let (status, model, duration, input_tok, output_tok): (String, String, i64, i64, i64) = conn
        .query_row(
            "SELECT status, model, duration_ms, input_tokens, output_tokens \
             FROM workflow_nodes WHERE run_id = ?1 AND node_id = ?2",
            ("run-1", "plan"),
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .expect("query");
    assert_eq!(status, "completed");
    assert_eq!(model, "gpt-5"); // COALESCE preserves first non-null
    assert_eq!(duration, 500);
    assert_eq!(input_tok, 100); // COALESCE preserves from first insert
    assert_eq!(output_tok, 200); // set by second upsert
}

#[test]
fn insert_edge() {
    let (_dir, backend) = temp_db();
    backend.insert_run("p", "", "").expect("insert run");
    backend
        .insert_edge(0, "start", "plan", Some("default"))
        .expect("insert edge");

    let conn = backend.connection().lock().expect("lock");
    let (from, to): (String, String) = conn
        .query_row(
            "SELECT from_node, to_node FROM workflow_edges WHERE run_id = ?1 AND step_index = 0",
            ("run-1",),
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("query");
    assert_eq!(from, "start");
    assert_eq!(to, "plan");
}

#[test]
fn definition_snapshot_dedup() {
    let (_dir, backend) = temp_db();
    backend.insert_run("p", "", "").expect("insert run");

    backend
        .save_definition_snapshot("abc123", b"content", "workflow", "test.md")
        .expect("first save");
    backend
        .save_definition_snapshot("abc123", b"different content", "workflow", "test.md")
        .expect("second save (ignored)");

    let conn = backend.connection().lock().expect("lock");
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM workflow_definition_snapshots WHERE content_hash = ?1",
            ("abc123",),
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(count, 1);
}

#[test]
fn save_and_load_node_output() {
    let (_dir, backend) = temp_db();
    backend.insert_run("p", "", "").expect("insert run");

    let output = b"Hello, this is the LLM output";
    backend
        .save_node_output("plan", output)
        .expect("save output");

    let conn = backend.connection().lock().expect("lock");
    let data: Vec<u8> = conn
        .query_row(
            "SELECT output FROM workflow_node_outputs WHERE run_id = ?1 AND node_id = ?2",
            ("run-1", "plan"),
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(data, output);
}

#[test]
fn delete_run_removes_run_scoped_rows() {
    let (_dir, backend) = temp_db();
    backend.insert_run("p", "", "").expect("insert run");
    backend
        .upsert_node(&NodeRecord {
            node_id: "plan",
            status: "success",
            model: None,
            provider: None,
            duration_ms: None,
            input_tokens: None,
            output_tokens: None,
            retry_count: Some(0),
            failure_reason: None,
        })
        .expect("insert node");
    backend
        .insert_edge(0, "start", "plan", None)
        .expect("insert edge");
    backend
        .save_node_output("plan", b"hello")
        .expect("save output");

    backend.delete_run().expect("delete run");

    let conn = backend.connection().lock().expect("lock");
    let run_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM workflow_runs WHERE run_id = ?1",
            ("run-1",),
            |row| row.get(0),
        )
        .expect("query runs");
    let node_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM workflow_nodes WHERE run_id = ?1",
            ("run-1",),
            |row| row.get(0),
        )
        .expect("query nodes");
    assert_eq!(run_count, 0);
    assert_eq!(node_count, 0);
}

// ---------------------------------------------------------------------------
// Concurrent access
// ---------------------------------------------------------------------------

#[test]
fn concurrent_reads_and_writes() {
    let (_dir, ctx) = temp_context();

    // Simulate concurrent access within a single thread
    ctx.set("counter", json!(0));

    for i in 1..=10 {
        ctx.set("counter", json!(i));
    }

    assert_eq!(ctx.get_i64("counter"), Some(10));
}

// ---------------------------------------------------------------------------
// Run isolation
// ---------------------------------------------------------------------------

#[test]
fn runs_are_isolated() {
    let dir = TempDir::new().expect("temp dir");
    let db_path = dir.path().join("test.db");
    let workspace_db = WorkspaceDb::open(&db_path).expect("open workspace db");

    let ctx1 = Context::with_sqlite(&workspace_db, "run-a").expect("ctx1");
    let ctx2 = Context::with_sqlite(&workspace_db, "run-b").expect("ctx2");

    // Insert run rows for foreign key constraints
    if let Some(b) = ctx1.sqlite_backend() {
        b.insert_run("p", "", "").expect("run a");
    }
    if let Some(b) = ctx2.sqlite_backend() {
        b.insert_run("p", "", "").expect("run b");
    }

    ctx1.set("key", json!("from-a"));
    ctx2.set("key", json!("from-b"));

    assert_eq!(ctx1.get_string("key"), "from-a");
    assert_eq!(ctx2.get_string("key"), "from-b");
}
