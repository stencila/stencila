//! State DB: a small mutable SQLite file that tracks document routing,
//! tombstones, and build metadata.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

use crate::error::Result;
use crate::segment::SegmentId;

/// The state database file name.
const STATE_FILE: &str = "state.sqlite";

// ---------------------------------------------------------------------------
// FileEntry — a tracked source file
// ---------------------------------------------------------------------------

/// Information about a previously-indexed source file.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub doc_id: String,
    pub path: PathBuf,
    pub file_hash: String,
    pub segment_id: SegmentId,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Mutable state database for the corpus.
pub struct State {
    conn: Connection,
}

impl State {
    /// Open (or create) the state database in the `.corpus/` directory.
    pub fn open(corpus_dir: &Path) -> Result<Self> {
        let path = corpus_dir.join(STATE_FILE);
        let conn = Connection::open(&path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS doc_segments (
                doc_id     TEXT PRIMARY KEY,
                path       TEXT NOT NULL,
                file_hash  TEXT NOT NULL,
                segment_id INTEGER NOT NULL,
                indexed_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tombstones (
                doc_id     TEXT PRIMARY KEY,
                segment_id INTEGER NOT NULL,
                deleted_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS build_meta (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )?;

        Ok(Self { conn })
    }

    /// Return all tracked files as a map from path → FileEntry.
    pub fn file_index(&self) -> Result<HashMap<PathBuf, FileEntry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT doc_id, path, file_hash, segment_id FROM doc_segments")?;

        let mut map = HashMap::new();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let path_str: String = row.get(1)?;
            let entry = FileEntry {
                doc_id: row.get(0)?,
                path: PathBuf::from(&path_str),
                file_hash: row.get(2)?,
                segment_id: SegmentId(row.get::<_, i64>(3)? as u64),
            };
            map.insert(entry.path.clone(), entry);
        }

        Ok(map)
    }

    /// Register a newly-indexed document.
    pub fn register_doc(
        &self,
        doc_id: &str,
        path: &Path,
        file_hash: &str,
        segment_id: SegmentId,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO doc_segments (doc_id, path, file_hash, segment_id, indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                doc_id,
                path.to_string_lossy().as_ref(),
                file_hash,
                segment_id.value() as i64,
                chrono::Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Mark a document as tombstoned (deleted or superseded).
    pub fn tombstone_doc(&self, doc_id: &str, segment_id: SegmentId) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "INSERT OR REPLACE INTO tombstones (doc_id, segment_id, deleted_at)
             VALUES (?1, ?2, ?3)",
            params![
                doc_id,
                segment_id.value() as i64,
                chrono::Utc::now().to_rfc3339(),
            ],
        )?;
        tx.execute("DELETE FROM doc_segments WHERE doc_id = ?1", params![doc_id])?;
        tx.commit()?;
        Ok(())
    }

    /// All tombstoned doc IDs (for query-time filtering).
    pub fn all_tombstoned_doc_ids(&self) -> Result<HashSet<String>> {
        let mut stmt = self.conn.prepare("SELECT doc_id FROM tombstones")?;
        let mut set = HashSet::new();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            set.insert(row.get(0)?);
        }
        Ok(set)
    }

    /// Get a build metadata value.
    pub fn get_meta(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM build_meta WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    /// Set a build metadata value.
    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO build_meta (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_open_register_tombstone_roundtrip() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let state = State::open(tmp.path()).expect("open state");

        // Register two documents
        state
            .register_doc("d1", Path::new("a.md"), "hash_a", SegmentId(1))
            .expect("register d1");
        state
            .register_doc("d2", Path::new("b.md"), "hash_b", SegmentId(1))
            .expect("register d2");

        let index = state.file_index().expect("file_index");
        assert_eq!(index.len(), 2);
        assert_eq!(index[&PathBuf::from("a.md")].doc_id, "d1");
        assert_eq!(index[&PathBuf::from("b.md")].file_hash, "hash_b");

        // Tombstone d1
        state.tombstone_doc("d1", SegmentId(1)).expect("tombstone");

        let index = state.file_index().expect("file_index after tombstone");
        assert_eq!(index.len(), 1);
        assert!(!index.contains_key(&PathBuf::from("a.md")));

        let tombstones = state.all_tombstoned_doc_ids().expect("tombstones");
        assert!(tombstones.contains("d1"));
        assert!(!tombstones.contains("d2"));
    }

    #[test]
    fn build_meta_roundtrip() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let state = State::open(tmp.path()).expect("open state");

        assert!(state.get_meta("last_build_at").expect("get").is_none());

        state
            .set_meta("last_build_at", "2026-02-07T00:00:00Z")
            .expect("set");
        assert_eq!(
            state.get_meta("last_build_at").expect("get"),
            Some("2026-02-07T00:00:00Z".into())
        );
    }
}
