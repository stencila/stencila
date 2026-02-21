//! Tests for [`WorkspaceDb`] and the migration runner.

use tempfile::TempDir;

use stencila_db::WorkspaceDb;
use stencila_db::migration::Migration;
use stencila_db::rusqlite;

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

// ---------------------------------------------------------------------------
// verify_databases tests
// ---------------------------------------------------------------------------

mod verify {
    use super::*;
    use stencila_db::sync::verify_databases;

    fn create_db_with_data(dir: &std::path::Path, name: &str, rows: &[(i32, &str)]) -> std::path::PathBuf {
        let path = dir.join(name);
        let db = WorkspaceDb::open(&path).expect("open");
        db.migrate(
            "test",
            &[Migration {
                version: 1,
                name: "create_items",
                sql: "CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
            }],
        )
        .expect("migrate");
        let conn = db.connection().lock().expect("lock");
        for &(id, name) in rows {
            conn.execute(
                "INSERT INTO items (id, name) VALUES (?1, ?2)",
                rusqlite::params![id, name],
            )
            .expect("insert");
        }
        path
    }

    #[test]
    fn identical_databases_match() {
        let dir = TempDir::new().expect("temp dir");
        let rows = &[(1, "alice"), (2, "bob")];
        let a = create_db_with_data(dir.path(), "a.sqlite3", rows);
        let b = create_db_with_data(dir.path(), "b.sqlite3", rows);

        let result = verify_databases(&a, &b).expect("verify");
        assert!(result.matches, "Identical databases should match");
        assert!(result.differences.is_empty());
    }

    #[test]
    fn different_row_counts_detected() {
        let dir = TempDir::new().expect("temp dir");
        let a = create_db_with_data(dir.path(), "a.sqlite3", &[(1, "alice"), (2, "bob")]);
        let b = create_db_with_data(dir.path(), "b.sqlite3", &[(1, "alice")]);

        let result = verify_databases(&a, &b).expect("verify");
        assert!(!result.matches);
        assert!(result.differences.iter().any(|d| d.contains("row count")));
    }

    #[test]
    fn different_content_detected() {
        let dir = TempDir::new().expect("temp dir");
        let a = create_db_with_data(dir.path(), "a.sqlite3", &[(1, "alice")]);
        let b = create_db_with_data(dir.path(), "b.sqlite3", &[(1, "bob")]);

        let result = verify_databases(&a, &b).expect("verify");
        assert!(!result.matches);
        assert!(result.differences.iter().any(|d| d.contains("content differs")));
    }

    #[test]
    fn missing_table_detected() {
        let dir = TempDir::new().expect("temp dir");

        // a has two tables, b has one
        let a_path = dir.path().join("a.sqlite3");
        let a_db = WorkspaceDb::open(&a_path).expect("open");
        a_db.migrate(
            "test",
            &[
                Migration {
                    version: 1,
                    name: "create_items",
                    sql: "CREATE TABLE items (id INTEGER PRIMARY KEY);",
                },
                Migration {
                    version: 2,
                    name: "create_extra",
                    sql: "CREATE TABLE extra (id INTEGER PRIMARY KEY);",
                },
            ],
        )
        .expect("migrate");

        let b_path = dir.path().join("b.sqlite3");
        let b_db = WorkspaceDb::open(&b_path).expect("open");
        b_db.migrate(
            "test",
            &[Migration {
                version: 1,
                name: "create_items",
                sql: "CREATE TABLE items (id INTEGER PRIMARY KEY);",
            }],
        )
        .expect("migrate");

        let result = verify_databases(&a_path, &b_path).expect("verify");
        assert!(!result.matches);
        assert!(result.differences.iter().any(|d| d.contains("extra")));
    }

    #[test]
    fn empty_databases_match() {
        let dir = TempDir::new().expect("temp dir");
        let a = create_db_with_data(dir.path(), "a.sqlite3", &[]);
        let b = create_db_with_data(dir.path(), "b.sqlite3", &[]);

        let result = verify_databases(&a, &b).expect("verify");
        assert!(result.matches);
    }

    #[test]
    fn schema_drift_detected() {
        let dir = TempDir::new().expect("temp dir");

        // a has items with (id, name), b has items with (id, name, email)
        let a_path = dir.path().join("a.sqlite3");
        let a_db = WorkspaceDb::open(&a_path).expect("open");
        a_db.migrate(
            "test",
            &[Migration {
                version: 1,
                name: "create_items",
                sql: "CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
            }],
        )
        .expect("migrate");

        let b_path = dir.path().join("b.sqlite3");
        let b_db = WorkspaceDb::open(&b_path).expect("open");
        b_db.migrate(
            "test",
            &[Migration {
                version: 1,
                name: "create_items_v2",
                sql: "CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT);",
            }],
        )
        .expect("migrate");

        let result = verify_databases(&a_path, &b_path).expect("verify");
        assert!(!result.matches);
        assert!(
            result.differences.iter().any(|d| d.contains("schema differs")),
            "Expected schema difference, got: {:?}",
            result.differences
        );
    }

    #[test]
    fn index_difference_detected() {
        let dir = TempDir::new().expect("temp dir");

        // a has an index, b doesn't
        let a_path = dir.path().join("a.sqlite3");
        let a_db = WorkspaceDb::open(&a_path).expect("open");
        a_db.migrate(
            "test",
            &[
                Migration {
                    version: 1,
                    name: "create_items",
                    sql: "CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
                },
                Migration {
                    version: 2,
                    name: "add_index",
                    sql: "CREATE INDEX idx_items_name ON items (name);",
                },
            ],
        )
        .expect("migrate");

        let b_path = dir.path().join("b.sqlite3");
        let b_db = WorkspaceDb::open(&b_path).expect("open");
        b_db.migrate(
            "test",
            &[Migration {
                version: 1,
                name: "create_items",
                sql: "CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
            }],
        )
        .expect("migrate");

        let result = verify_databases(&a_path, &b_path).expect("verify");
        assert!(!result.matches);
        assert!(
            result.differences.iter().any(|d| d.contains("indexes differ")),
            "Expected index difference, got: {:?}",
            result.differences
        );
    }
}
