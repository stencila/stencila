pub mod migration;
pub mod sync;

pub use rusqlite;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, TransactionBehavior};

use crate::migration::Migration;

/// A shared workspace database connection.
///
/// Wraps `Arc<Mutex<rusqlite::Connection>>` with connection setup
/// (WAL mode, foreign keys, busy timeout) and a per-domain migration runner.
///
/// The database file lives at `.stencila/db.sqlite3` in the workspace root.
/// Multiple domains (workflows, chat sessions, etc.) share the same database,
/// each owning their own set of tables managed via namespaced migrations.
pub struct WorkspaceDb {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl WorkspaceDb {
    /// Open (or create) a workspace database at `db_path`.
    ///
    /// Enables WAL mode for concurrent read access, turns on foreign key
    /// enforcement, and sets a 5-second busy timeout. Creates the
    /// `_migrations` tracking table if it does not exist.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if the database cannot be opened or
    /// initial setup fails.
    pub fn open(db_path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;

        // Create the migrations tracking table.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                domain     TEXT NOT NULL,
                version    INTEGER NOT NULL,
                name       TEXT NOT NULL,
                applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                PRIMARY KEY (domain, version)
            )",
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: db_path.to_path_buf(),
        })
    }

    /// Get the shared database connection handle.
    ///
    /// Consumer crates capture this `Arc` to perform domain-specific
    /// operations on the shared database.
    #[must_use]
    pub fn connection(&self) -> &Arc<Mutex<Connection>> {
        &self.conn
    }

    /// The path to the database file.
    #[must_use]
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Run pending migrations for a domain.
    ///
    /// Migrations are applied in version order within a single IMMEDIATE
    /// transaction. The version check and migration application are atomic,
    /// so concurrent processes racing on first startup will serialize
    /// correctly — one acquires the write lock, the other waits and then
    /// sees the already-applied state.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if any migration fails. On failure, the
    /// entire batch is rolled back.
    pub fn migrate(&self, domain: &str, migrations: &[Migration]) -> Result<(), rusqlite::Error> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // Use IMMEDIATE to acquire a write lock before reading, preventing
        // two processes from both deciding the same migration is pending.
        let tx = rusqlite::Transaction::new_unchecked(&conn, TransactionBehavior::Immediate)?;

        let current_version: i32 = tx.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _migrations WHERE domain = ?1",
            (domain,),
            |row| row.get(0),
        )?;

        let pending: Vec<&Migration> = migrations
            .iter()
            .filter(|m| m.version > current_version)
            .collect();

        if pending.is_empty() {
            return Ok(());
        }

        for migration in &pending {
            tx.execute_batch(migration.sql)?;
            // INSERT OR IGNORE as belt-and-suspenders: if another process
            // somehow slipped through, we don't fail on PK conflict.
            tx.execute(
                "INSERT OR IGNORE INTO _migrations (domain, version, name) VALUES (?1, ?2, ?3)",
                (domain, migration.version, migration.name),
            )?;
        }
        tx.commit()?;

        tracing::debug!(
            "Applied {} migration(s) for domain `{domain}` (up to version {})",
            pending.len(),
            pending.last().map_or(current_version, |m| m.version),
        );

        Ok(())
    }
}
