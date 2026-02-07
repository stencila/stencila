//! Segment file abstraction.
//!
//! A **segment** is a single, self-contained SQLite file. Once **sealed** it is
//! never modified again — making it content-addressable and sync-friendly.

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OpenFlags};

use crate::error::{Error, Result};
use crate::schema;

// ---------------------------------------------------------------------------
// SegmentId
// ---------------------------------------------------------------------------

/// Opaque segment identifier. Monotonically increasing per corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SegmentId(pub(crate) u64);

impl SegmentId {
    /// Format as the filename component: `seg_000001`.
    pub fn filename(&self) -> String {
        format!("seg_{:06}", self.0)
    }

    /// Full SQLite filename: `seg_000001.sqlite`.
    pub fn sqlite_filename(&self) -> String {
        format!("{}.sqlite", self.filename())
    }

    /// Resolve the full path for this segment within a segments directory.
    pub fn path_in(&self, segments_dir: &Path) -> PathBuf {
        segments_dir.join(self.sqlite_filename())
    }

    /// Numeric value.
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for SegmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// ChunkRow
// ---------------------------------------------------------------------------

/// A single row to be written into a segment's `chunks` table.
///
/// This is intentionally simple — the schema layer will define richer chunk
/// types later. This struct carries exactly the columns that `schema::init`
/// creates.
#[derive(Debug, Clone)]
pub struct ChunkRow {
    pub chunk_id: String,
    pub doc_id: String,
    pub path: String,
    pub node_type: String,
    pub text: String,
    pub metadata: String,
}

// ---------------------------------------------------------------------------
// Segment
// ---------------------------------------------------------------------------

/// A single segment SQLite database.
pub struct Segment {
    pub(crate) id: SegmentId,
    pub(crate) path: PathBuf,
    pub(crate) conn: Connection,
    pub(crate) sealed: bool,
}

impl Segment {
    /// Create a new **active** (read-write) segment at the given path.
    ///
    /// Runs schema initialization and enables WAL mode for crash safety.
    pub fn create(segments_dir: &Path, id: SegmentId) -> Result<Self> {
        let path = id.path_in(segments_dir);

        let conn = Connection::open(&path)?;

        // WAL mode for crash safety during writes.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;

        schema::init(&conn)?;

        Ok(Self {
            id,
            path,
            conn,
            sealed: false,
        })
    }

    /// Open an existing segment **read-only** (for queries on sealed segments).
    pub fn open_readonly(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(Error::SegmentNotFound(path.to_path_buf()));
        }

        let conn =
            Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        // Try to read segment id from the filename (seg_NNNNNN.sqlite).
        let id = id_from_path(path);

        Ok(Self {
            id,
            path: path.to_path_buf(),
            conn,
            sealed: true,
        })
    }

    /// Segment identifier.
    pub fn id(&self) -> SegmentId {
        self.id
    }

    /// Whether this segment has been sealed.
    pub fn is_sealed(&self) -> bool {
        self.sealed
    }

    // -- Writing (active segments only) ------------------------------------

    /// Insert a batch of chunk rows into this segment.
    ///
    /// Uses a transaction for atomicity and performance.
    pub fn write_batch(&self, rows: &[ChunkRow]) -> Result<()> {
        if self.sealed {
            return Err(Error::SegmentSealed(self.id.0));
        }
        if rows.is_empty() {
            return Ok(());
        }

        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR REPLACE INTO chunks (chunk_id, doc_id, path, node_type, text, metadata)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for row in rows {
                stmt.execute(params![
                    row.chunk_id,
                    row.doc_id,
                    row.path,
                    row.node_type,
                    row.text,
                    row.metadata,
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Register a source document in the segment's `docs` table.
    pub fn register_doc(&self, doc_id: &str, path: &str, hash: &str) -> Result<()> {
        if self.sealed {
            return Err(Error::SegmentSealed(self.id.0));
        }

        self.conn.execute(
            "INSERT OR REPLACE INTO docs (doc_id, path, hash, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![doc_id, path, hash, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    // -- Sealing -----------------------------------------------------------

    /// Seal this segment: vacuum, switch to journal_mode=OFF, compute blake3 hash.
    ///
    /// After this call the segment is read-only. Returns the sealed metadata.
    pub fn seal(&mut self) -> Result<SealedMeta> {
        if self.sealed {
            return Err(Error::SegmentSealed(self.id.0));
        }

        // Optimize before sealing.
        self.conn.execute_batch("PRAGMA optimize;")?;
        self.conn.execute_batch("VACUUM;")?;
        // Switch off journaling — sealed segments are never written to.
        self.conn.pragma_update(None, "journal_mode", "DELETE")?;

        // Close the connection so the file is fully flushed.
        // We reopen read-only below.
        let path = self.path.clone();
        // Drop old connection by replacing it with a throwaway in-memory one,
        // then reopen the file read-only.
        self.conn = Connection::open_in_memory()?;

        // Delete any leftover WAL/SHM files.
        let wal = path.with_extension("sqlite-wal");
        let shm = path.with_extension("sqlite-shm");
        if wal.exists() {
            std::fs::remove_file(&wal)?;
        }
        if shm.exists() {
            std::fs::remove_file(&shm)?;
        }

        let hash = hash_file(&path)?;
        let size = std::fs::metadata(&path)?.len();

        // Reopen read-only.
        self.conn =
            Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        self.sealed = true;

        Ok(SealedMeta {
            id: self.id,
            hash,
            size,
            sealed_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    // -- Stats -------------------------------------------------------------

    /// Number of chunks in this segment.
    pub fn chunk_count(&self) -> Result<u64> {
        let count: i64 = self
            .conn
            .query_row("SELECT count(*) FROM chunks", [], |row| row.get(0))?;
        Ok(count as u64)
    }

    /// File size of the segment in bytes.
    pub fn file_size(&self) -> Result<u64> {
        Ok(std::fs::metadata(&self.path)?.len())
    }

    // -- Querying ----------------------------------------------------------

    /// Execute a full-text search and return matching chunk rows.
    ///
    /// This is the per-segment query primitive used by the parallel query engine.
    pub fn fts_search(
        &self,
        query_text: &str,
        node_types: &[String],
        limit: usize,
    ) -> Result<Vec<QueryHit>> {
        // Build the SQL dynamically based on filters.
        let mut sql = String::from(
            "SELECT c.chunk_id, c.doc_id, c.path, c.node_type, c.text, c.metadata,
                    rank
             FROM chunks_fts f
             JOIN chunks c ON c.rowid = f.rowid
             WHERE chunks_fts MATCH ?1",
        );

        if !node_types.is_empty() {
            let placeholders: Vec<String> = node_types
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 2))
                .collect();
            sql.push_str(&format!(
                " AND c.node_type IN ({})",
                placeholders.join(", ")
            ));
        }

        sql.push_str(" ORDER BY rank LIMIT ?");
        // The limit param index is after the node_type params.
        let limit_idx = node_types.len() + 2;
        sql = sql.replace(
            "LIMIT ?",
            &format!("LIMIT ?{limit_idx}"),
        );

        let mut stmt = self.conn.prepare(&sql)?;

        // Bind parameters.
        stmt.raw_bind_parameter(1, query_text)?;
        for (i, nt) in node_types.iter().enumerate() {
            stmt.raw_bind_parameter(i + 2, nt.as_str())?;
        }
        stmt.raw_bind_parameter(limit_idx, limit as i64)?;

        let mut hits = Vec::new();
        let mut rows = stmt.raw_query();
        while let Some(row) = rows.next()? {
            hits.push(QueryHit {
                segment_id: self.id,
                chunk_id: row.get(0)?,
                doc_id: row.get(1)?,
                path: row.get(2)?,
                node_type: row.get(3)?,
                text: row.get(4)?,
                metadata: row.get(5)?,
                score: {
                    let rank: f64 = row.get(6)?;
                    // FTS5 rank is negative (lower = better), invert for sorting.
                    -rank
                },
            });
        }

        Ok(hits)
    }
}

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// Metadata produced when a segment is sealed.
#[derive(Debug, Clone)]
pub struct SealedMeta {
    pub id: SegmentId,
    pub hash: String,
    pub size: u64,
    pub sealed_at: String,
}

/// A single query hit from a segment.
#[derive(Debug, Clone)]
pub struct QueryHit {
    pub segment_id: SegmentId,
    pub chunk_id: String,
    pub doc_id: String,
    pub path: String,
    pub node_type: String,
    pub text: String,
    pub metadata: String,
    pub score: f64,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compute the blake3 hash of a file.
fn hash_file(path: &Path) -> Result<String> {
    let data = std::fs::read(path)?;
    Ok(blake3::hash(&data).to_hex().to_string())
}

/// Try to extract a SegmentId from a path like `seg_000001.sqlite`.
fn id_from_path(path: &Path) -> SegmentId {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("seg_000000");
    let num = stem.strip_prefix("seg_").unwrap_or("0");
    let n = num.parse::<u64>().unwrap_or(0);
    SegmentId(n)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_id_formatting() {
        let id = SegmentId(1);
        assert_eq!(id.sqlite_filename(), "seg_000001.sqlite");
        assert_eq!(id.filename(), "seg_000001");

        let id = SegmentId(999_999);
        assert_eq!(id.sqlite_filename(), "seg_999999.sqlite");
    }

    #[test]
    fn create_write_seal_query_lifecycle() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dir = tmp.path();

        // Create
        let mut seg = Segment::create(dir, SegmentId(1)).expect("create segment");
        assert!(!seg.is_sealed());

        // Write
        let rows = vec![
            ChunkRow {
                chunk_id: "c1".into(),
                doc_id: "d1".into(),
                path: "test.md".into(),
                node_type: "Paragraph".into(),
                text: "The quick brown fox jumps over the lazy dog".into(),
                metadata: "{}".into(),
            },
            ChunkRow {
                chunk_id: "c2".into(),
                doc_id: "d1".into(),
                path: "test.md".into(),
                node_type: "Sentence".into(),
                text: "Protein folding is important for biology".into(),
                metadata: "{}".into(),
            },
        ];
        seg.write_batch(&rows).expect("write batch");
        assert_eq!(seg.chunk_count().expect("count"), 2);

        // Seal
        let meta = seg.seal().expect("seal");
        assert!(seg.is_sealed());
        assert!(!meta.hash.is_empty());
        assert!(meta.size > 0);

        // Writing after seal should fail
        assert!(seg.write_batch(&rows).is_err());

        // Open read-only and query
        let ro_path = SegmentId(1).path_in(dir);
        let ro = Segment::open_readonly(&ro_path).expect("open ro");
        assert!(ro.is_sealed());

        let hits = ro
            .fts_search("protein folding", &[], 10)
            .expect("fts search");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].chunk_id, "c2");

        // Query with node_type filter
        let hits = ro
            .fts_search("protein folding", &["Paragraph".into()], 10)
            .expect("fts search with filter");
        assert_eq!(hits.len(), 0);

        let hits = ro
            .fts_search("protein folding", &["Sentence".into()], 10)
            .expect("fts search sentence filter");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn id_from_path_parses_correctly() {
        let id = id_from_path(Path::new("/foo/segments/seg_000042.sqlite"));
        assert_eq!(id.value(), 42);

        let id = id_from_path(Path::new("unknown.sqlite"));
        assert_eq!(id.value(), 0);
    }
}
