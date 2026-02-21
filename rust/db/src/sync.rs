use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::Utc;
use eyre::{Context, Result, bail};
use rusqlite::Connection;
use rusqlite::session::{ConflictAction, ConflictType, Session};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Manifest types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub format: String,
    pub schema_version: BTreeMap<String, i32>,
    pub base_snapshot: SnapshotEntry,
    #[serde(default)]
    pub changesets: Vec<ChangesetEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotEntry {
    pub hash: String,
    pub compression: String,
    pub schema_version: BTreeMap<String, i32>,
    pub created_at: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesetEntry {
    pub hash: String,
    pub schema_version: BTreeMap<String, i32>,
    pub created_at: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub const MANIFEST_FORMAT: &str = "stencila-db-sync-v1";
pub const MANIFEST_FILE: &str = "db.json";

/// Maximum number of changesets before automatic snapshot rotation on push.
pub const AUTO_SNAPSHOT_CHANGESET_LIMIT: usize = 50;

/// Cumulative changeset size (bytes) that triggers automatic snapshot rotation.
pub const AUTO_SNAPSHOT_SIZE_LIMIT: u64 = 50 * 1024 * 1024; // 50 MB

// ---------------------------------------------------------------------------
// Manifest I/O
// ---------------------------------------------------------------------------

pub fn read_manifest(manifest_path: &Path) -> Result<Option<Manifest>> {
    if !manifest_path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(manifest_path).context("reading manifest")?;
    let manifest: Manifest = serde_json::from_str(&data).context("parsing manifest")?;
    if manifest.format != MANIFEST_FORMAT {
        bail!(
            "Unsupported manifest format `{}`; expected `{MANIFEST_FORMAT}`",
            manifest.format
        );
    }
    Ok(Some(manifest))
}

pub fn write_manifest(manifest_path: &Path, manifest: &Manifest) -> Result<()> {
    let data = serde_json::to_string_pretty(manifest).context("serializing manifest")?;
    fs::write(manifest_path, data).context("writing manifest")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Content addressing
// ---------------------------------------------------------------------------

pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

// ---------------------------------------------------------------------------
// Zstd compression
// ---------------------------------------------------------------------------

pub fn zstd_compress(data: &[u8]) -> Result<Vec<u8>> {
    zstd::encode_all(data, 3).context("zstd compress")
}

pub fn zstd_decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = zstd::Decoder::new(data).context("zstd decoder init")?;
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .context("zstd decompress read")?;
    Ok(out)
}

// ---------------------------------------------------------------------------
// _sync table
// ---------------------------------------------------------------------------

pub fn ensure_sync_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _sync (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
    )
    .context("create _sync table")?;
    Ok(())
}

pub fn get_sync_position(conn: &Connection) -> Result<Option<String>> {
    ensure_sync_table(conn)?;
    let mut stmt = conn.prepare("SELECT value FROM _sync WHERE key = 'head'")?;
    let mut rows = stmt.query([])?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get(0)?)),
        None => Ok(None),
    }
}

pub fn set_sync_position(conn: &Connection, hash: &str) -> Result<()> {
    ensure_sync_table(conn)?;
    conn.execute(
        "INSERT OR REPLACE INTO _sync (key, value) VALUES ('head', ?1)",
        [hash],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Schema version from _migrations
// ---------------------------------------------------------------------------

pub fn schema_versions(conn: &Connection) -> Result<BTreeMap<String, i32>> {
    let mut map = BTreeMap::new();
    let mut stmt =
        conn.prepare("SELECT domain, COALESCE(MAX(version), 0) FROM _migrations GROUP BY domain")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let domain: String = row.get(0)?;
        let version: i32 = row.get(1)?;
        map.insert(domain, version);
    }
    Ok(map)
}

// ---------------------------------------------------------------------------
// Local blob cache
// ---------------------------------------------------------------------------

pub fn cache_path(stencila_dir: &Path, kind: &str, hash: &str) -> PathBuf {
    stencila_dir.join("cache").join("db").join(kind).join(hash)
}

pub fn ensure_cache_dir(stencila_dir: &Path, kind: &str) -> Result<PathBuf> {
    let dir = stencila_dir.join("cache").join("db").join(kind);
    fs::create_dir_all(&dir).context("create cache dir")?;
    Ok(dir)
}

pub fn read_cached_blob(stencila_dir: &Path, kind: &str, hash: &str) -> Option<Vec<u8>> {
    let path = cache_path(stencila_dir, kind, hash);
    if path.exists() {
        tracing::debug!("Using cached {kind} blob {hash}");
        let data = fs::read(&path).ok()?;
        // Verify integrity — cached blobs could be corrupted or tampered with
        if sha256_hex(&data) != hash {
            tracing::warn!("Cached {kind} blob {hash} failed integrity check, discarding");
            fs::remove_file(&path).ok();
            return None;
        }
        Some(data)
    } else {
        None
    }
}

pub fn write_cached_blob(stencila_dir: &Path, kind: &str, hash: &str, data: &[u8]) -> Result<()> {
    ensure_cache_dir(stencila_dir, kind)?;
    fs::write(cache_path(stencila_dir, kind, hash), data).context("write blob to cache")
}

/// Remove cached blobs that are not referenced by the given manifest.
///
/// Returns `(removed_count, freed_bytes)`.
pub fn clean_blob_cache(stencila_dir: &Path, manifest: &Manifest) -> Result<(usize, u64)> {
    let mut referenced = std::collections::HashSet::new();
    referenced.insert(manifest.base_snapshot.hash.clone());
    for cs in &manifest.changesets {
        referenced.insert(cs.hash.clone());
    }

    let mut removed = 0usize;
    let mut freed = 0u64;

    for kind in &["snapshots", "changesets"] {
        let dir = stencila_dir.join("cache").join("db").join(kind);
        if !dir.exists() {
            continue;
        }
        let entries = fs::read_dir(&dir).context("read cache dir")?;
        for entry in entries {
            let entry = entry.context("read cache entry")?;
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();
            if !referenced.contains(name.as_ref()) {
                let meta = entry.metadata().ok();
                let size = meta.map(|m| m.len()).unwrap_or(0);
                if fs::remove_file(entry.path()).is_ok() {
                    removed += 1;
                    freed += size;
                }
            }
        }
    }

    Ok((removed, freed))
}

/// Check whether adding one more changeset to this manifest would reach
/// the automatic snapshot rotation threshold (by count or cumulative size).
///
/// The check accounts for the pending changeset that is about to be
/// appended, so with a limit of 50, this returns `true` when there are
/// already 49 changesets (the next one would be #50).
pub fn should_auto_snapshot(manifest: &Manifest) -> bool {
    if manifest.changesets.len() + 1 >= AUTO_SNAPSHOT_CHANGESET_LIMIT {
        return true;
    }
    let total_size: u64 = manifest.changesets.iter().map(|c| c.size).sum();
    total_size >= AUTO_SNAPSHOT_SIZE_LIMIT
}

// ---------------------------------------------------------------------------
// SQL identifier quoting
// ---------------------------------------------------------------------------

/// Quote a SQL identifier for safe interpolation. Doubles any embedded
/// double-quote characters per the SQL standard: `a"b` → `"a""b"`.
fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

// ---------------------------------------------------------------------------
// Changeset generation via Session::diff
// ---------------------------------------------------------------------------

fn user_table_names(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE '\\_%' ESCAPE '\\' ORDER BY name",
    )?;
    let names: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(names)
}

/// Reconstruct the manifest head state into a temp database file.
///
/// Restores the base snapshot and applies all existing changesets in order.
/// The caller must supply the raw blob bytes for each entry (fetched from
/// cache or cloud).
///
/// Returns the path to the temp file. The caller is responsible for
/// removing it when done.
pub fn reconstruct_head(
    dest: &Path,
    snapshot_blob: &[u8],
    compression: &str,
    changesets: &[(&str, &[u8])],
) -> Result<()> {
    restore_snapshot(snapshot_blob, compression, dest, "reconstruct")?;

    for &(hash, data) in changesets {
        apply_changeset(dest, data, hash)?;
    }

    Ok(())
}

/// Compute a changeset by diffing the live database against a
/// reconstructed manifest head. Returns `None` if there are no
/// differences (nothing to push).
///
/// `head_path` must be an already-reconstructed database representing
/// the current manifest head state.
pub fn create_changeset(db_path: &Path, head_path: &Path) -> Result<Option<Vec<u8>>> {
    let conn = Connection::open(db_path)?;

    // Escape single quotes in path for safe SQL interpolation
    let escaped = head_path.display().to_string().replace('\'', "''");
    conn.execute_batch(&format!("ATTACH DATABASE '{escaped}' AS head"))
        .context("attach head database")?;

    let tables = user_table_names(&conn)?;

    let mut session = Session::new(&conn)?;
    for table in &tables {
        session
            .diff::<&str, &str>("head", table.as_str())
            .with_context(|| format!("diff table `{table}`"))?;
    }

    if session.is_empty() {
        return Ok(None);
    }

    let mut buf = Vec::new();
    session.changeset_strm(&mut buf)?;

    if buf.is_empty() {
        Ok(None)
    } else {
        Ok(Some(buf))
    }
}

// ---------------------------------------------------------------------------
// WAL helpers
// ---------------------------------------------------------------------------

fn wal_checkpoint(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
        .context("WAL checkpoint")?;
    Ok(())
}

/// Copy a database file to `dest`, checkpointing the WAL first so the
/// main file is self-contained. This avoids silently losing WAL content
/// that hasn't been flushed to the main file.
pub fn checkpoint_and_copy(src: &Path, dest: &Path) -> Result<()> {
    let conn = Connection::open(src)?;
    wal_checkpoint(&conn)?;
    drop(conn);
    fs::copy(src, dest).context("copy database file")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Create a snapshot (full DB file, zstd-compressed)
// ---------------------------------------------------------------------------

pub fn create_snapshot(db_path: &Path) -> Result<Vec<u8>> {
    // Checkpoint WAL so the main file is self-contained
    {
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
    }
    let raw = fs::read(db_path).context("read database file")?;
    zstd_compress(&raw)
}

// ---------------------------------------------------------------------------
// Restore a snapshot into a database file
// ---------------------------------------------------------------------------

pub fn restore_snapshot(
    snap_data: &[u8],
    compression: &str,
    dest: &Path,
    sync_hash: &str,
) -> Result<()> {
    let raw = match compression {
        "zstd" => zstd_decompress(snap_data)?,
        "none" => snap_data.to_vec(),
        other => bail!("Unsupported compression: {other}"),
    };

    fs::write(dest, &raw).context("write database file")?;

    let conn = Connection::open(dest)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    ensure_sync_table(&conn)?;
    set_sync_position(&conn, sync_hash)?;

    // Checkpoint so the main file is self-contained before a rename
    wal_checkpoint(&conn)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Apply a changeset blob to a database file
// ---------------------------------------------------------------------------

pub fn apply_changeset(db_path: &Path, cs_data: &[u8], cs_hash: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    let mut input = cs_data;
    conn.apply_strm(
        &mut input,
        None::<fn(&str) -> bool>,
        |conflict_type, _item| match conflict_type {
            // Remote wins for data conflicts
            ConflictType::SQLITE_CHANGESET_DATA => ConflictAction::SQLITE_CHANGESET_REPLACE,
            // Row already deleted/missing — skip, the intent is satisfied
            ConflictType::SQLITE_CHANGESET_NOTFOUND => ConflictAction::SQLITE_CHANGESET_OMIT,
            // CONSTRAINT and FOREIGN_KEY indicate a bug — surface as error
            _ => ConflictAction::SQLITE_CHANGESET_ABORT,
        },
    )
    .context("apply changeset")?;

    set_sync_position(&conn, cs_hash)?;

    // Checkpoint so the main file is self-contained before a rename
    wal_checkpoint(&conn)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Build a new manifest after a snapshot push
// ---------------------------------------------------------------------------

pub fn new_snapshot_manifest(
    hash: String,
    size: u64,
    schema_version: BTreeMap<String, i32>,
    message: Option<String>,
) -> Manifest {
    let now = Utc::now().to_rfc3339();
    Manifest {
        format: MANIFEST_FORMAT.to_string(),
        schema_version: schema_version.clone(),
        base_snapshot: SnapshotEntry {
            hash,
            compression: "zstd".to_string(),
            schema_version,
            created_at: now,
            size,
            message,
        },
        changesets: vec![],
    }
}

/// Create a new changeset entry to append to an existing manifest.
pub fn new_changeset_entry(
    hash: String,
    size: u64,
    schema_version: BTreeMap<String, i32>,
    message: Option<String>,
) -> ChangesetEntry {
    ChangesetEntry {
        hash,
        schema_version,
        created_at: Utc::now().to_rfc3339(),
        size,
        message,
    }
}

// ---------------------------------------------------------------------------
// Logical database comparison (for verify)
// ---------------------------------------------------------------------------

/// Result of comparing two databases logically (table-by-table, row-by-row).
pub struct VerifyResult {
    pub matches: bool,
    pub differences: Vec<String>,
}

/// Compare two database files logically. Both are queried table-by-table and
/// row digests are compared. Internal SQLite page layout makes file-level
/// comparison unreliable.
pub fn verify_databases(local_path: &Path, reference_path: &Path) -> Result<VerifyResult> {
    let local = Connection::open(local_path)?;
    let reference = Connection::open(reference_path)?;

    let local_tables = user_table_names(&local)?;
    let ref_tables = user_table_names(&reference)?;

    let mut diffs = Vec::new();

    // Check for tables in local but not in reference
    for t in &local_tables {
        if !ref_tables.contains(t) {
            diffs.push(format!(
                "Table `{t}` exists locally but not in manifest state"
            ));
        }
    }

    // Check for tables in reference but not in local
    for t in &ref_tables {
        if !local_tables.contains(t) {
            diffs.push(format!(
                "Table `{t}` exists in manifest state but not locally"
            ));
        }
    }

    // Compare schema definitions for shared tables (DDL, indexes, triggers)
    let shared: Vec<&String> = local_tables
        .iter()
        .filter(|t| ref_tables.contains(t))
        .collect();

    for table in &shared {
        // Compare table DDL (column definitions, constraints)
        let local_ddl = table_ddl(&local, table)?;
        let ref_ddl = table_ddl(&reference, table)?;
        if local_ddl != ref_ddl {
            diffs.push(format!(
                "Table `{table}`: schema differs (local: `{local_ddl}`, manifest: `{ref_ddl}`)"
            ));
        }

        // Compare indexes on this table
        let local_indexes = table_indexes(&local, table)?;
        let ref_indexes = table_indexes(&reference, table)?;
        if local_indexes != ref_indexes {
            let missing: Vec<_> = ref_indexes
                .iter()
                .filter(|(name, _)| !local_indexes.iter().any(|(n, _)| n == name))
                .map(|(n, _)| n.as_str())
                .collect();
            let extra: Vec<_> = local_indexes
                .iter()
                .filter(|(name, _)| !ref_indexes.iter().any(|(n, _)| n == name))
                .map(|(n, _)| n.as_str())
                .collect();
            let altered: Vec<_> = local_indexes
                .iter()
                .filter(|(name, sql)| ref_indexes.iter().any(|(n, s)| n == name && s != sql))
                .map(|(n, _)| n.as_str())
                .collect();
            let mut parts = Vec::new();
            if !missing.is_empty() {
                parts.push(format!("missing: {}", missing.join(", ")));
            }
            if !extra.is_empty() {
                parts.push(format!("extra: {}", extra.join(", ")));
            }
            if !altered.is_empty() {
                parts.push(format!("altered: {}", altered.join(", ")));
            }
            diffs.push(format!(
                "Table `{table}`: indexes differ ({})",
                parts.join("; ")
            ));
        }

        // Compare triggers on this table
        let local_triggers = table_triggers(&local, table)?;
        let ref_triggers = table_triggers(&reference, table)?;
        if local_triggers != ref_triggers {
            diffs.push(format!("Table `{table}`: triggers differ"));
        }
    }

    // Compare row data for shared tables
    for table in &shared {
        // Compare row counts first (fast path)
        let qt = quote_ident(table);
        let local_count: i64 =
            local.query_row(&format!("SELECT COUNT(*) FROM {qt}"), [], |row| row.get(0))?;
        let ref_count: i64 =
            reference.query_row(&format!("SELECT COUNT(*) FROM {qt}"), [], |row| row.get(0))?;

        if local_count != ref_count {
            diffs.push(format!(
                "Table `{table}`: row count differs (local: {local_count}, manifest: {ref_count})"
            ));
            continue;
        }

        if local_count == 0 {
            continue;
        }

        // Compare content via a hash of all rows.
        // We serialize each row as a group_concat of all columns and hash the full output.
        let local_hash = table_content_hash(&local, table)?;
        let ref_hash = table_content_hash(&reference, table)?;

        if local_hash != ref_hash {
            diffs.push(format!(
                "Table `{table}`: content differs ({local_count} rows)"
            ));
        }
    }

    Ok(VerifyResult {
        matches: diffs.is_empty(),
        differences: diffs,
    })
}

fn table_ddl(conn: &Connection, table: &str) -> Result<String> {
    let sql: String = conn.query_row(
        "SELECT sql FROM sqlite_master WHERE type='table' AND name = ?1",
        [table],
        |row| row.get(0),
    )?;
    Ok(sql)
}

fn table_indexes(conn: &Connection, table: &str) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT name, COALESCE(sql, '') FROM sqlite_master \
         WHERE type='index' AND tbl_name = ?1 \
         ORDER BY name",
    )?;
    let indexes: Vec<(String, String)> = stmt
        .query_map([table], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(indexes)
}

fn table_triggers(conn: &Connection, table: &str) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT name, COALESCE(sql, '') FROM sqlite_master \
         WHERE type='trigger' AND tbl_name = ?1 \
         ORDER BY name",
    )?;
    let triggers: Vec<(String, String)> = stmt
        .query_map([table], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(triggers)
}

fn table_content_hash(conn: &Connection, table: &str) -> Result<String> {
    // Get column names to build a deterministic query
    let qt = quote_ident(table);
    let mut col_stmt = conn.prepare(&format!("PRAGMA table_info({qt})"))?;
    let columns: Vec<String> = col_stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    if columns.is_empty() {
        return Ok(String::new());
    }

    // Build a query that concatenates all columns with a separator, ordered deterministically.
    // We use quote() to handle NULLs and binary data consistently.
    let col_exprs: Vec<String> = columns
        .iter()
        .map(|c| format!("quote({})", quote_ident(c)))
        .collect();
    let concat_expr = col_exprs.join(" || '|' || ");

    let query = format!(
        "SELECT {concat_expr} FROM {qt} ORDER BY {}",
        columns
            .iter()
            .map(|c| quote_ident(c))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut hasher = Sha256::new();
    let mut stmt = conn.prepare(&query)?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let val: String = row.get(0)?;
        hasher.update(val.as_bytes());
        hasher.update(b"\n");
    }

    Ok(format!("{:x}", hasher.finalize()))
}

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

pub struct SyncStatus {
    pub db_exists: bool,
    pub db_size: Option<u64>,
    pub schema_version: BTreeMap<String, i32>,
    pub manifest_exists: bool,
    pub base_snapshot_hash: Option<String>,
    pub base_snapshot_date: Option<String>,
    pub base_snapshot_size: Option<u64>,
    pub total_changesets: usize,
    pub applied_changesets: usize,
    pub sync_head: Option<String>,
    pub up_to_date: bool,
    /// Local `_sync` position exists but is not found in the current
    /// manifest's history (base snapshot or any changeset). This indicates
    /// a branch rewind or divergence — the user needs `stencila db reset`.
    pub diverged: bool,
}

pub fn status(stencila_dir: &Path, db_path: &Path) -> Result<SyncStatus> {
    let manifest_path = stencila_dir.join(MANIFEST_FILE);
    let manifest = read_manifest(&manifest_path)?;

    let db_exists = db_path.exists();
    let db_size = if db_exists {
        Some(fs::metadata(db_path)?.len())
    } else {
        None
    };

    let (schema_version, sync_head) = if db_exists {
        let conn = Connection::open(db_path)?;
        let sv = schema_versions(&conn)?;
        let head = get_sync_position(&conn)?;
        (sv, head)
    } else {
        (BTreeMap::new(), None)
    };

    match manifest {
        Some(m) => {
            let target = m
                .changesets
                .last()
                .map(|c| c.hash.as_str())
                .unwrap_or(&m.base_snapshot.hash);

            let (applied, diverged) = if let Some(ref head) = sync_head {
                if *head == m.base_snapshot.hash {
                    (0, false)
                } else {
                    match m.changesets.iter().position(|c| c.hash == *head) {
                        Some(i) => (i + 1, false),
                        // sync_head exists but is not in the manifest history
                        None => (0, true),
                    }
                }
            } else {
                (0, false)
            };

            Ok(SyncStatus {
                db_exists,
                db_size,
                schema_version,
                manifest_exists: true,
                base_snapshot_hash: Some(m.base_snapshot.hash.clone()),
                base_snapshot_date: Some(m.base_snapshot.created_at.clone()),
                base_snapshot_size: Some(m.base_snapshot.size),
                total_changesets: m.changesets.len(),
                applied_changesets: applied,
                up_to_date: !diverged && sync_head.as_deref() == Some(target),
                sync_head,
                diverged,
            })
        }
        None => Ok(SyncStatus {
            db_exists,
            db_size,
            schema_version,
            manifest_exists: false,
            base_snapshot_hash: None,
            base_snapshot_date: None,
            base_snapshot_size: None,
            total_changesets: 0,
            applied_changesets: 0,
            sync_head,
            up_to_date: false,
            diverged: false,
        }),
    }
}
