use std::env::current_dir;
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use eyre::Result;

use stencila_cli_utils::color_print::cstr;
use stencila_cli_utils::message;
use stencila_db::rusqlite;
use stencila_db::sync;

/// Manage the workspace database
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show sync status</dim>
  <b>stencila db status</b>

  <dim># Push database state to cloud</dim>
  <b>stencila db push</b>

  <dim># Pull database state from cloud</dim>
  <b>stencila db pull</b>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    Push(Push),
    Pull(Pull),
    Status(Status),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Command::Push(push) => push.run().await,
            Command::Pull(pull) => pull.run().await,
            Command::Status(status) => status.run().await,
        }
    }
}

// ---------------------------------------------------------------------------
// Resolve stencila dir and db path
// ---------------------------------------------------------------------------

async fn resolve_paths() -> Result<(PathBuf, PathBuf)> {
    let stencila_dir = stencila_dirs::closest_stencila_dir(&current_dir()?, false).await?;
    let db_path = stencila_dir.join(stencila_dirs::DB_SQLITE_FILE);
    Ok((stencila_dir, db_path))
}

fn workspace_id() -> Result<String> {
    let cfg = stencila_config::get()?;
    cfg.workspace
        .and_then(|w| w.id)
        .ok_or_else(|| eyre::eyre!(
            "No workspace.id configured. Run `stencila init` first or set workspace.id in stencila.toml."
        ))
}

// ---------------------------------------------------------------------------
// push
// ---------------------------------------------------------------------------

/// Push database state to Stencila Cloud
#[derive(Debug, Args)]
#[command(after_long_help = PUSH_AFTER_LONG_HELP)]
pub struct Push {
    /// Optional message describing this push
    #[arg(short, long)]
    message: Option<String>,
}

pub static PUSH_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Push current database state</dim>
  <b>stencila db push</b>

  <dim># Push with a description</dim>
  <b>stencila db push -m \"add batch-1 results\"</b>
"
);

impl Push {
    pub async fn run(self) -> Result<()> {
        let (stencila_dir, db_path) = resolve_paths().await?;
        let ws_id = workspace_id()?;

        if !db_path.exists() {
            eyre::bail!("No database at `{}`; nothing to push", db_path.display());
        }

        let manifest_path = stencila_dir.join(sync::MANIFEST_FILE);
        let existing = sync::read_manifest(&manifest_path)?;

        // Read local schema version
        let conn = rusqlite::Connection::open(&db_path)?;
        sync::ensure_sync_table(&conn)?;
        let local_schema = sync::schema_versions(&conn)?;
        drop(conn);

        // Decide: snapshot or changeset?
        //   - First push (no manifest) → snapshot
        //   - Schema changed → snapshot
        //   - Otherwise → changeset via Session::diff against baseline
        let needs_snapshot = match &existing {
            None => true,
            Some(m) => m.schema_version != local_schema,
        };

        if needs_snapshot {
            if let Some(ref m) = existing {
                if m.schema_version != local_schema {
                    tracing::info!("Schema changed, creating snapshot instead of changeset");
                }
            }
            self.push_snapshot(
                &stencila_dir,
                &db_path,
                &manifest_path,
                &ws_id,
                local_schema,
            )
            .await
        } else {
            self.push_changeset(
                &stencila_dir,
                &db_path,
                &manifest_path,
                &ws_id,
                local_schema,
                existing.unwrap(),
            )
            .await
        }
    }

    async fn push_snapshot(
        &self,
        stencila_dir: &std::path::Path,
        db_path: &std::path::Path,
        manifest_path: &std::path::Path,
        ws_id: &str,
        local_schema: std::collections::BTreeMap<String, i32>,
    ) -> Result<()> {
        let compressed = sync::create_snapshot(db_path)?;
        let hash = sync::sha256_hex(&compressed);
        let size = compressed.len() as u64;

        // Cache before uploading (upload consumes the Vec)
        sync::write_cached_blob(stencila_dir, "snapshots", &hash, &compressed)?;

        // Upload blob before writing manifest (invariant §5)
        stencila_cloud::db::upload_blob(ws_id, "snapshots", &hash, compressed).await?;

        let manifest =
            sync::new_snapshot_manifest(hash.clone(), size, local_schema, self.message.clone());
        sync::write_manifest(manifest_path, &manifest)?;

        let conn = rusqlite::Connection::open(db_path)?;
        sync::set_sync_position(&conn, &hash)?;

        message!(
            "📦 Pushed snapshot {:.8} ({}). Remember to `git add .stencila/db.json`",
            hash,
            format_bytes(size)
        );
        Ok(())
    }

    async fn push_changeset(
        &self,
        stencila_dir: &std::path::Path,
        db_path: &std::path::Path,
        manifest_path: &std::path::Path,
        ws_id: &str,
        local_schema: std::collections::BTreeMap<String, i32>,
        mut manifest: sync::Manifest,
    ) -> Result<()> {
        // Reconstruct the manifest head (snapshot + all existing changesets)
        // into a temp database, so we diff against the true current state
        // rather than just the base snapshot.
        let head_path = db_path.with_extension("head.tmp");

        // Guard: ensure head.tmp is cleaned up even if we return early on error
        let _cleanup = TempFileGuard::new(head_path.clone());

        let snapshot_blob = fetch_blob(
            stencila_dir,
            ws_id,
            "snapshots",
            &manifest.base_snapshot.hash,
        )
        .await?;

        let mut cs_blobs = Vec::new();
        for entry in &manifest.changesets {
            let data = fetch_blob(stencila_dir, ws_id, "changesets", &entry.hash).await?;
            cs_blobs.push((entry.hash.clone(), data));
        }

        let cs_refs: Vec<(&str, &[u8])> = cs_blobs
            .iter()
            .map(|(h, d)| (h.as_str(), d.as_slice()))
            .collect();

        sync::reconstruct_head(
            &head_path,
            &snapshot_blob,
            &manifest.base_snapshot.compression,
            &cs_refs,
        )?;

        // Diff local db against the reconstructed head
        let cs_bytes = sync::create_changeset(db_path, &head_path)?;

        let Some(cs_bytes) = cs_bytes else {
            message!("✅ Nothing to push — database is unchanged");
            return Ok(());
        };

        let hash = sync::sha256_hex(&cs_bytes);
        let size = cs_bytes.len() as u64;

        // Cache before uploading (upload consumes the Vec)
        sync::write_cached_blob(stencila_dir, "changesets", &hash, &cs_bytes)?;

        // Upload blob before writing manifest (invariant §5)
        stencila_cloud::db::upload_blob(ws_id, "changesets", &hash, cs_bytes).await?;

        // Append changeset to manifest
        let entry =
            sync::new_changeset_entry(hash.clone(), size, local_schema, self.message.clone());
        manifest.changesets.push(entry);
        sync::write_manifest(manifest_path, &manifest)?;

        let conn = rusqlite::Connection::open(db_path)?;
        sync::set_sync_position(&conn, &hash)?;

        message!(
            "📝 Pushed changeset {:.8} ({}). Remember to `git add .stencila/db.json`",
            hash,
            format_bytes(size)
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// pull
// ---------------------------------------------------------------------------

/// Pull database state from Stencila Cloud
#[derive(Debug, Args)]
#[command(after_long_help = PULL_AFTER_LONG_HELP)]
pub struct Pull {
    /// Force pull even when local database has diverged from the manifest
    #[arg(long)]
    force: bool,
}

pub static PULL_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Pull latest database state</dim>
  <b>stencila db pull</b>

  <dim># Force pull when local state has diverged</dim>
  <b>stencila db pull --force</b>
"
);

impl Pull {
    pub async fn run(self) -> Result<()> {
        let (stencila_dir, db_path) = resolve_paths().await?;
        let ws_id = workspace_id()?;

        let manifest_path = stencila_dir.join(sync::MANIFEST_FILE);
        let manifest = sync::read_manifest(&manifest_path)?.ok_or_else(|| {
            eyre::eyre!(
                "No manifest at `{}`; nothing to pull",
                manifest_path.display()
            )
        })?;

        // Determine current local sync position
        let local_head = if db_path.exists() {
            let conn = rusqlite::Connection::open(&db_path)?;
            sync::get_sync_position(&conn)?
        } else {
            None
        };

        let target_head = manifest
            .changesets
            .last()
            .map(|c| c.hash.as_str())
            .unwrap_or(&manifest.base_snapshot.hash);

        if local_head.as_deref() == Some(target_head) {
            message!("✅ Already up to date");
            return Ok(());
        }

        // Determine if we need the base snapshot
        let need_snapshot = match &local_head {
            None => true,
            Some(h) => {
                *h != manifest.base_snapshot.hash
                    && !manifest.changesets.iter().any(|c| c.hash == *h)
            }
        };

        // Divergence: local head exists but isn't in the manifest
        if need_snapshot && local_head.is_some() {
            if !self.force {
                eyre::bail!(
                    "Local database has diverged from the manifest (local sync position not found \
                     in manifest). Run `stencila db push` to preserve local changes first, or \
                     `stencila db pull --force` to discard local state and rebuild from manifest."
                );
            }
            tracing::warn!(
                "Local sync position does not match manifest — performing full restore from snapshot"
            );
        }

        // Build into a temp file for atomicity (invariant §4)
        let tmp_path = db_path.with_extension("sqlite3.tmp");

        // Clean up stale temp files from a previous crashed pull, and
        // ensure cleanup on error exit
        let _cleanup = TempFileGuard::new(tmp_path.clone());

        if need_snapshot {
            let snap_data = fetch_blob(
                &stencila_dir,
                &ws_id,
                "snapshots",
                &manifest.base_snapshot.hash,
            )
            .await?;
            sync::restore_snapshot(
                &snap_data,
                &manifest.base_snapshot.compression,
                &tmp_path,
                &manifest.base_snapshot.hash,
            )?;
        } else {
            // Checkpoint WAL before copying so the main file is self-contained
            sync::checkpoint_and_copy(&db_path, &tmp_path)?;
        }

        // Run migrations so the temp database schema matches what the
        // current code expects before applying changesets (design §
        // "Migrations and changesets" — restore, then migrate, then apply).
        {
            let tmp_db = stencila_db::WorkspaceDb::open(&tmp_path)
                .map_err(|e| eyre::eyre!("Failed to open temp database for migration: {e}"))?;
            stencila_workflows::run_migrations(&tmp_db)
                .map_err(|e| eyre::eyre!("Failed to run migrations on temp database: {e}"))?;
        }

        // Apply changesets after current position
        let start_idx = if need_snapshot {
            0
        } else {
            let local = local_head.as_deref().unwrap_or("");
            if local == manifest.base_snapshot.hash {
                0
            } else {
                manifest
                    .changesets
                    .iter()
                    .position(|c| c.hash == local)
                    .map(|i| i + 1)
                    .unwrap_or(0)
            }
        };

        let pending = &manifest.changesets[start_idx..];
        for entry in pending {
            let cs_data = fetch_blob(&stencila_dir, &ws_id, "changesets", &entry.hash).await?;
            sync::apply_changeset(&tmp_path, &cs_data, &entry.hash)?;
        }

        // Atomic replace (invariant §4)
        std::fs::rename(&tmp_path, &db_path).map_err(|e| eyre::eyre!("atomic replace: {e}"))?;

        if need_snapshot {
            message!(
                "📦 Restored snapshot + applied {} changeset(s)",
                pending.len()
            );
        } else {
            message!("📝 Applied {} changeset(s)", pending.len());
        }

        Ok(())
    }
}

/// Fetch a blob using the local cache, falling back to cloud download.
async fn fetch_blob(
    stencila_dir: &std::path::Path,
    workspace_id: &str,
    kind: &str,
    hash: &str,
) -> Result<Vec<u8>> {
    if let Some(data) = sync::read_cached_blob(stencila_dir, kind, hash) {
        return Ok(data);
    }
    let data = stencila_cloud::db::download_blob(workspace_id, kind, hash).await?;
    sync::write_cached_blob(stencila_dir, kind, hash, &data)?;
    Ok(data)
}

// ---------------------------------------------------------------------------
// status
// ---------------------------------------------------------------------------

/// Show database sync status
#[derive(Debug, Args)]
#[command(after_long_help = STATUS_AFTER_LONG_HELP)]
pub struct Status {}

pub static STATUS_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show sync status</dim>
  <b>stencila db status</b>
"
);

impl Status {
    pub async fn run(self) -> Result<()> {
        let (stencila_dir, db_path) = resolve_paths().await?;
        let s = sync::status(&stencila_dir, &db_path)?;

        if s.db_exists {
            println!(
                "Workspace database: .stencila/{} ({})",
                stencila_dirs::DB_SQLITE_FILE,
                format_bytes(s.db_size.unwrap_or(0))
            );
        } else {
            println!("Workspace database: not found");
        }

        if !s.schema_version.is_empty() {
            let domains: Vec<String> = s
                .schema_version
                .iter()
                .map(|(d, v)| format!("{d}@{v}"))
                .collect();
            println!("Domains: {}", domains.join(", "));
        }

        println!();

        if s.manifest_exists {
            println!("Sync:");
            if let Some(ref hash) = s.base_snapshot_hash {
                let date = s
                    .base_snapshot_date
                    .as_deref()
                    .and_then(|d| d.get(..10))
                    .unwrap_or("unknown");
                let size = format_bytes(s.base_snapshot_size.unwrap_or(0));
                println!("  Base snapshot: {:.8} ({date}, {size})", hash);
            }
            println!(
                "  Applied changesets: {} of {}",
                s.applied_changesets, s.total_changesets
            );
            if s.up_to_date {
                println!("  Status: up to date");
            } else if !s.db_exists {
                println!("  Status: run `stencila db pull` to restore");
            } else {
                println!("  Status: behind — run `stencila db pull`");
            }
        } else {
            println!("Sync: no manifest (run `stencila db push` to initialize)");
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

struct TempFileGuard(PathBuf);

impl TempFileGuard {
    fn new(path: PathBuf) -> Self {
        // Eagerly clean up stale files from a previous crash
        Self::remove_all(&path);
        Self(path)
    }

    fn remove_all(path: &Path) {
        for ext in &["", "-wal", "-shm"] {
            let mut p = path.as_os_str().to_owned();
            p.push(ext);
            std::fs::remove_file(PathBuf::from(p)).ok();
        }
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        Self::remove_all(&self.0);
    }
}

fn format_bytes(n: u64) -> String {
    if n < 1024 {
        format!("{n} B")
    } else if n < 1024 * 1024 {
        format!("{:.1} KB", n as f64 / 1024.0)
    } else if n < 1024 * 1024 * 1024 {
        format!("{:.1} MB", n as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", n as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
