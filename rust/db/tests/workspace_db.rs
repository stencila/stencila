//! Tests for [`WorkspaceDb`] and the migration runner.

use tempfile::TempDir;

use stencila_db::WorkspaceDb;
use stencila_db::migration::Migration;

fn temp_workspace_db() -> (TempDir, WorkspaceDb) {
    let dir = TempDir::new().expect("temp dir");
    let db_path = dir.path().join("db.sqlite3");
    let db = WorkspaceDb::open(&db_path).expect("open");
    (dir, db)
}

#[test]
fn open_creates_migrations_table() {
    let (_dir, db) = temp_workspace_db();
    let conn = db.connection().lock().expect("lock");
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migrations'",
            [],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(count, 1);
}

#[test]
fn open_is_idempotent() {
    let dir = TempDir::new().expect("temp dir");
    let db_path = dir.path().join("db.sqlite3");
    let _db1 = WorkspaceDb::open(&db_path).expect("first open");
    let _db2 = WorkspaceDb::open(&db_path).expect("second open");
}

#[test]
fn migrate_applies_pending() {
    let (_dir, db) = temp_workspace_db();

    let migrations = [
        Migration {
            version: 1,
            name: "create_foo",
            sql: "CREATE TABLE foo (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
        },
        Migration {
            version: 2,
            name: "create_bar",
            sql: "CREATE TABLE bar (id INTEGER PRIMARY KEY);",
        },
    ];

    db.migrate("test_domain", &migrations).expect("migrate");

    let conn = db.connection().lock().expect("lock");

    // Both tables should exist
    let table_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('foo', 'bar')",
            [],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(table_count, 2);

    // Two migration records should exist
    let migration_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE domain = 'test_domain'",
            [],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(migration_count, 2);
}

#[test]
fn migrate_skips_already_applied() {
    let (_dir, db) = temp_workspace_db();

    let migrations_v1 = [Migration {
        version: 1,
        name: "create_foo",
        sql: "CREATE TABLE foo (id INTEGER PRIMARY KEY);",
    }];

    db.migrate("test_domain", &migrations_v1)
        .expect("first migrate");

    // Add a second migration and re-run — only v2 should be applied
    let migrations_v1_v2 = [
        Migration {
            version: 1,
            name: "create_foo",
            sql: "CREATE TABLE foo (id INTEGER PRIMARY KEY);",
        },
        Migration {
            version: 2,
            name: "create_bar",
            sql: "CREATE TABLE bar (id INTEGER PRIMARY KEY);",
        },
    ];

    db.migrate("test_domain", &migrations_v1_v2)
        .expect("second migrate");

    let conn = db.connection().lock().expect("lock");
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE domain = 'test_domain'",
            [],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(count, 2);
}

#[test]
fn multi_domain_migrations() {
    let (_dir, db) = temp_workspace_db();

    let domain_a = [Migration {
        version: 1,
        name: "create_a",
        sql: "CREATE TABLE domain_a (id INTEGER PRIMARY KEY);",
    }];

    let domain_b = [Migration {
        version: 1,
        name: "create_b",
        sql: "CREATE TABLE domain_b (id INTEGER PRIMARY KEY);",
    }];

    db.migrate("domain_a", &domain_a).expect("migrate a");
    db.migrate("domain_b", &domain_b).expect("migrate b");

    let conn = db.connection().lock().expect("lock");

    // Both domain tables exist
    let table_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('domain_a', 'domain_b')",
            [],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(table_count, 2);

    // Each domain has its own migration record
    let a_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE domain = 'domain_a'",
            [],
            |row| row.get(0),
        )
        .expect("query");
    let b_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE domain = 'domain_b'",
            [],
            |row| row.get(0),
        )
        .expect("query");
    assert_eq!(a_count, 1);
    assert_eq!(b_count, 1);
}

#[test]
fn db_path_is_recorded() {
    let dir = TempDir::new().expect("temp dir");
    let db_path = dir.path().join("db.sqlite3");
    let db = WorkspaceDb::open(&db_path).expect("open");
    assert_eq!(db.db_path(), db_path);
}
