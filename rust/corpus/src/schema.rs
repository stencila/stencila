//! Segment schema initialization.
//!
//! Defines the DDL applied to every new segment SQLite database.
//! This is a placeholder — the real schema (chunk fields, FTS config,
//! citation tables, etc.) will be iterated separately.

use rusqlite::Connection;

use crate::error::Result;

/// Current schema version string. Bump when the DDL changes.
pub const SCHEMA_VERSION: &str = "0.1.0";

/// Initialize the segment schema on a freshly-created SQLite database.
///
/// This creates the minimal tables needed for the segment lifecycle to
/// work (write chunks, query, seal). The actual column set will evolve.
pub fn init(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- Core chunks table: one row per indexable unit.
        CREATE TABLE IF NOT EXISTS chunks (
            chunk_id   TEXT PRIMARY KEY,
            doc_id     TEXT NOT NULL,
            path       TEXT NOT NULL,
            node_type  TEXT NOT NULL,
            text       TEXT NOT NULL DEFAULT '',
            metadata   TEXT NOT NULL DEFAULT '{}'
        );

        CREATE INDEX IF NOT EXISTS idx_chunks_doc_id ON chunks(doc_id);
        CREATE INDEX IF NOT EXISTS idx_chunks_node_type ON chunks(node_type);

        -- FTS5 virtual table over the text column.
        CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
            text,
            chunk_id UNINDEXED,
            doc_id UNINDEXED,
            node_type UNINDEXED,
            content=chunks,
            content_rowid=rowid
        );

        -- Triggers to keep FTS in sync with the chunks table.
        CREATE TRIGGER IF NOT EXISTS chunks_ai AFTER INSERT ON chunks BEGIN
            INSERT INTO chunks_fts(rowid, text, chunk_id, doc_id, node_type)
            VALUES (new.rowid, new.text, new.chunk_id, new.doc_id, new.node_type);
        END;

        -- Docs table: one row per source document in this segment.
        CREATE TABLE IF NOT EXISTS docs (
            doc_id     TEXT PRIMARY KEY,
            path       TEXT NOT NULL,
            hash       TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        ",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_init_succeeds() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        init(&conn).expect("schema init");

        // Verify chunks table exists
        let count: i64 = conn
            .query_row("SELECT count(*) FROM chunks", [], |row| row.get(0))
            .expect("query chunks");
        assert_eq!(count, 0);

        // Verify FTS table exists
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM chunks_fts WHERE chunks_fts MATCH 'test'",
                [],
                |row| row.get(0),
            )
            .expect("query fts");
        assert_eq!(count, 0);
    }
}
