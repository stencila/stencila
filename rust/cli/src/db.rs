use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::{Args, Parser, Subcommand};
use eyre::Result;
use stencila_ask::{Answer, AskOptions, ask_with};
use stencila_cli_utils::color_print::cstr;
use stencila_cli_utils::message;
use stencila_cli_utils::progress::{new_bytes_bar, new_items_bar, new_spinner};
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

  <dim># Show changeset history</dim>
  <b>stencila db log</b>

  <dim># Verify local db matches manifest</dim>
  <b>stencila db verify</b>

  <dim># Rebuild database from manifest</dim>
  <b>stencila db reset</b>

  <dim># Create a new baseline snapshot</dim>
  <b>stencila db snapshot</b>

  <dim># Clean local blob cache</dim>
  <b>stencila db clean</b>

  <dim># Remove orphaned remote blobs</dim>
  <b>stencila db gc</b>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    Push(Push),
    Pull(Pull),
    Status(Status),
    Log(Log),
    Verify(Verify),
    Reset(Reset),
    Snapshot(Snapshot),
    Clean(Clean),
    Gc(Gc),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Command::Push(push) => push.run().await,
            Command::Pull(pull) => pull.run().await,
            Command::Status(status) => status.run().await,
            Command::Log(log) => log.run().await,
            Command::Verify(verify) => verify.run().await,
            Command::Reset(reset) => reset.run().await,
            Command::Snapshot(snapshot) => snapshot.run().await,
            Command::Clean(clean) => clean.run().await,
            Command::Gc(gc) => gc.run().await,
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
            if let Some(ref m) = existing
                && m.schema_version != local_schema
            {
                message!(
                    "⚡ Schema changed ({} → {}), creating snapshot instead of changeset",
                    format_schema_version(&m.schema_version),
                    format_schema_version(&local_schema)
                );
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
                existing.expect("existing manifest when not needs_snapshot"),
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
        let spinner = new_spinner("Compressing snapshot...");
        let compressed = sync::create_snapshot(db_path)?;
        let hash = sync::sha256_hex(&compressed);
        let size = compressed.len() as u64;
        spinner.finish_and_clear();

        // Cache before uploading (upload consumes the Vec)
        sync::write_cached_blob(stencila_dir, "snapshots", &hash, &compressed)?;

        // Upload blob before writing manifest (invariant §5)
        let pb = new_bytes_bar(size, "Uploading snapshot");
        let pb_ref = pb.clone();
        stencila_cloud::db::upload_blob_with_progress(
            ws_id,
            "snapshots",
            &hash,
            compressed,
            Some(Arc::new(move |sent| pb_ref.set_position(sent))),
        )
        .await?;
        pb.finish_and_clear();

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

        let spinner = new_spinner("Reconstructing manifest head...");

        let snapshot_blob = fetch_blob(
            stencila_dir,
            ws_id,
            "snapshots",
            &manifest.base_snapshot.hash,
        )
        .await?;

        let total_cs = manifest.changesets.len();
        let mut cs_blobs = Vec::new();
        for (i, entry) in manifest.changesets.iter().enumerate() {
            spinner.set_message(format!("Fetching changeset {}/{}...", i + 1, total_cs));
            let data = fetch_blob(stencila_dir, ws_id, "changesets", &entry.hash).await?;
            cs_blobs.push((entry.hash.clone(), data));
        }

        spinner.set_message("Rebuilding head database...");

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
        spinner.set_message("Computing diff...");
        let cs_bytes = sync::create_changeset(db_path, &head_path)?;
        spinner.finish_and_clear();

        let Some(cs_bytes) = cs_bytes else {
            message!("✅ Nothing to push — database is unchanged");
            return Ok(());
        };

        // Auto-snapshot rotation: if adding this changeset would cross
        // the threshold, take a snapshot instead. This check runs after
        // diff so that "no changes" is always detected first.
        if sync::should_auto_snapshot(&manifest) {
            message!(
                "⚡ Auto-rotating to snapshot ({} changesets, {} cumulative)",
                manifest.changesets.len(),
                format_bytes(manifest.changesets.iter().map(|c| c.size).sum::<u64>())
            );
            return self
                .push_snapshot(stencila_dir, db_path, manifest_path, ws_id, local_schema)
                .await;
        }

        let hash = sync::sha256_hex(&cs_bytes);
        let size = cs_bytes.len() as u64;

        // Cache before uploading (upload consumes the Vec)
        sync::write_cached_blob(stencila_dir, "changesets", &hash, &cs_bytes)?;

        // Upload blob before writing manifest (invariant §5)
        let pb = new_bytes_bar(size, "Uploading changeset");
        let pb_ref = pb.clone();
        stencila_cloud::db::upload_blob_with_progress(
            ws_id,
            "changesets",
            &hash,
            cs_bytes,
            Some(Arc::new(move |sent| pb_ref.set_position(sent))),
        )
        .await?;
        pb.finish_and_clear();

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
            let pb = new_bytes_bar(manifest.base_snapshot.size, "Downloading snapshot");
            let pb_ref = pb.clone();
            let snap_data = fetch_blob_with_progress(
                &stencila_dir,
                &ws_id,
                "snapshots",
                &manifest.base_snapshot.hash,
                Some(Arc::new(move |recv, _total| pb_ref.set_position(recv))),
            )
            .await?;
            pb.finish_and_clear();
            let spinner = new_spinner("Restoring snapshot...");
            sync::restore_snapshot(
                &snap_data,
                &manifest.base_snapshot.compression,
                &tmp_path,
                &manifest.base_snapshot.hash,
            )?;
            spinner.finish_and_clear();
        } else {
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
        if !pending.is_empty() {
            let pb = new_items_bar(pending.len() as u64, "changesets");
            for entry in pending {
                let cs_data = fetch_blob(&stencila_dir, &ws_id, "changesets", &entry.hash).await?;
                sync::apply_changeset(&tmp_path, &cs_data, &entry.hash)?;
                pb.inc(1);
            }
            pb.finish_and_clear();
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
    fetch_blob_with_progress(stencila_dir, workspace_id, kind, hash, None).await
}

type ProgressCallback = Option<Arc<dyn Fn(u64, Option<u64>) + Send + Sync>>;

/// Fetch a blob with an optional progress callback for the download.
///
/// The callback receives `(bytes_received, total_bytes)`.
async fn fetch_blob_with_progress(
    stencila_dir: &std::path::Path,
    workspace_id: &str,
    kind: &str,
    hash: &str,
    on_progress: ProgressCallback,
) -> Result<Vec<u8>> {
    if let Some(data) = sync::read_cached_blob(stencila_dir, kind, hash) {
        return Ok(data);
    }
    let data =
        stencila_cloud::db::download_blob_with_progress(workspace_id, kind, hash, on_progress)
            .await?;
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
    #[allow(clippy::print_stdout)]
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
            if s.diverged {
                println!("  Applied changesets: unknown (diverged)");
                println!(
                    "  Status: ⚠ diverged — local sync position not in manifest. \
                     Run `stencila db reset` to rebuild, or `stencila db push` to \
                     preserve local changes first."
                );
            } else {
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
            }
        } else {
            println!("Sync: no manifest (run `stencila db push` to initialize)");
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// log
// ---------------------------------------------------------------------------

/// Show changeset history from the manifest
#[derive(Debug, Args)]
#[command(after_long_help = LOG_AFTER_LONG_HELP)]
pub struct Log {}

pub static LOG_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show changeset history</dim>
  <b>stencila db log</b>
"
);

impl Log {
    #[allow(clippy::print_stdout)]
    pub async fn run(self) -> Result<()> {
        let (stencila_dir, _db_path) = resolve_paths().await?;
        let manifest_path = stencila_dir.join(sync::MANIFEST_FILE);
        let manifest = sync::read_manifest(&manifest_path)?.ok_or_else(|| {
            eyre::eyre!(
                "No manifest at `{}`; nothing to show",
                manifest_path.display()
            )
        })?;

        // Print changesets in reverse chronological order
        for (i, entry) in manifest.changesets.iter().enumerate().rev() {
            let idx = i + 2; // #1 is the snapshot
            let date = entry.created_at.get(..16).unwrap_or(&entry.created_at);
            let msg = entry
                .message
                .as_deref()
                .map(|m| format!("  \"{m}\""))
                .unwrap_or_default();
            println!(
                "#{idx:<3} {:.8}  {date}  {}{msg}",
                entry.hash,
                format_bytes(entry.size)
            );
        }

        // Print the base snapshot as #1
        let snap = &manifest.base_snapshot;
        let date = snap.created_at.get(..16).unwrap_or(&snap.created_at);
        let msg = snap
            .message
            .as_deref()
            .map(|m| format!("  \"{m}\""))
            .unwrap_or_default();
        println!(
            "#1   [snapshot] {:.8}  {date}  {}{msg}",
            snap.hash,
            format_bytes(snap.size)
        );

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// verify
// ---------------------------------------------------------------------------

/// Verify local database matches the manifest state
#[derive(Debug, Args)]
#[command(after_long_help = VERIFY_AFTER_LONG_HELP)]
pub struct Verify {}

pub static VERIFY_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Verify local database matches manifest</dim>
  <b>stencila db verify</b>
"
);

impl Verify {
    #[allow(clippy::print_stdout)]
    pub async fn run(self) -> Result<()> {
        let (stencila_dir, db_path) = resolve_paths().await?;
        let ws_id = workspace_id()?;

        if !db_path.exists() {
            eyre::bail!("No database at `{}`; nothing to verify", db_path.display());
        }

        let manifest_path = stencila_dir.join(sync::MANIFEST_FILE);
        let manifest = sync::read_manifest(&manifest_path)?.ok_or_else(|| {
            eyre::eyre!(
                "No manifest at `{}`; nothing to verify against",
                manifest_path.display()
            )
        })?;

        // Reconstruct manifest head into a temp file
        let ref_path = db_path.with_extension("verify.tmp");
        let _cleanup = TempFileGuard::new(ref_path.clone());

        let pb = new_bytes_bar(manifest.base_snapshot.size, "Downloading snapshot");
        let pb_ref = pb.clone();
        let snapshot_blob = fetch_blob_with_progress(
            &stencila_dir,
            &ws_id,
            "snapshots",
            &manifest.base_snapshot.hash,
            Some(Arc::new(move |recv, _total| pb_ref.set_position(recv))),
        )
        .await?;
        pb.finish_and_clear();

        let spinner = new_spinner("Restoring snapshot...");
        sync::restore_snapshot(
            &snapshot_blob,
            &manifest.base_snapshot.compression,
            &ref_path,
            &manifest.base_snapshot.hash,
        )?;

        {
            let ref_db = stencila_db::WorkspaceDb::open(&ref_path)
                .map_err(|e| eyre::eyre!("Failed to open reference database: {e}"))?;
            stencila_workflows::run_migrations(&ref_db)
                .map_err(|e| eyre::eyre!("Failed to run migrations on reference database: {e}"))?;
        }
        spinner.finish_and_clear();

        if !manifest.changesets.is_empty() {
            let pb = new_items_bar(manifest.changesets.len() as u64, "changesets");
            for entry in &manifest.changesets {
                let cs_data = fetch_blob(&stencila_dir, &ws_id, "changesets", &entry.hash).await?;
                sync::apply_changeset(&ref_path, &cs_data, &entry.hash)?;
                pb.inc(1);
            }
            pb.finish_and_clear();
        }

        let spinner = new_spinner("Comparing databases...");
        let result = sync::verify_databases(&db_path, &ref_path)?;
        spinner.finish_and_clear();

        // Also check sync position validity — verify_databases intentionally
        // excludes internal tables (_sync, _migrations), so data can match
        // while the sync position is invalid for the current manifest.
        let s = sync::status(&stencila_dir, &db_path)?;

        let mut problems = result.differences;
        if s.diverged {
            problems.push(
                "Sync position: local _sync head is not in the current manifest \
                 (diverged — likely a branch switch or rewind)"
                    .to_string(),
            );
        }

        if problems.is_empty() {
            message!("✅ Local database matches manifest state");
            Ok(())
        } else {
            println!("❌ Local database does not match manifest state:");
            for diff in &problems {
                println!("  • {diff}");
            }
            println!();
            eyre::bail!(
                "Verification failed ({} difference{}). Run `stencila db reset` to rebuild from the manifest.",
                problems.len(),
                if problems.len() == 1 { "" } else { "s" }
            )
        }
    }
}

// ---------------------------------------------------------------------------
// reset
// ---------------------------------------------------------------------------

/// Rebuild local database from the manifest
///
/// Discards the local database and rebuilds it from scratch using the
/// manifest (snapshot + all changesets). This is the escape hatch when
/// the local database has diverged or become corrupted.
#[derive(Debug, Args)]
#[command(after_long_help = RESET_AFTER_LONG_HELP)]
pub struct Reset {}

pub static RESET_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Rebuild database from manifest</dim>
  <b>stencila db reset</b>
"
);

impl Reset {
    pub async fn run(self) -> Result<()> {
        let (stencila_dir, db_path) = resolve_paths().await?;
        let ws_id = workspace_id()?;

        let manifest_path = stencila_dir.join(sync::MANIFEST_FILE);
        let manifest = sync::read_manifest(&manifest_path)?.ok_or_else(|| {
            eyre::eyre!(
                "No manifest at `{}`; nothing to reset from",
                manifest_path.display()
            )
        })?;

        // Build into a temp file for atomicity (invariant §4)
        let tmp_path = db_path.with_extension("sqlite3.tmp");
        let _cleanup = TempFileGuard::new(tmp_path.clone());

        let pb = new_bytes_bar(manifest.base_snapshot.size, "Downloading snapshot");
        let pb_ref = pb.clone();
        let snap_data = fetch_blob_with_progress(
            &stencila_dir,
            &ws_id,
            "snapshots",
            &manifest.base_snapshot.hash,
            Some(Arc::new(move |recv, _total| pb_ref.set_position(recv))),
        )
        .await?;
        pb.finish_and_clear();
        let spinner = new_spinner("Restoring snapshot...");
        sync::restore_snapshot(
            &snap_data,
            &manifest.base_snapshot.compression,
            &tmp_path,
            &manifest.base_snapshot.hash,
        )?;
        spinner.finish_and_clear();

        // Run migrations so the schema matches current code before applying changesets
        {
            let tmp_db = stencila_db::WorkspaceDb::open(&tmp_path)
                .map_err(|e| eyre::eyre!("Failed to open temp database for migration: {e}"))?;
            stencila_workflows::run_migrations(&tmp_db)
                .map_err(|e| eyre::eyre!("Failed to run migrations on temp database: {e}"))?;
        }

        // Apply all changesets
        let cs_count = manifest.changesets.len();
        if cs_count > 0 {
            let pb = new_items_bar(cs_count as u64, "changesets");
            for entry in &manifest.changesets {
                let cs_data = fetch_blob(&stencila_dir, &ws_id, "changesets", &entry.hash).await?;
                sync::apply_changeset(&tmp_path, &cs_data, &entry.hash)?;
                pb.inc(1);
            }
            pb.finish_and_clear();
        }

        // Atomic replace (invariant §4)
        std::fs::rename(&tmp_path, &db_path).map_err(|e| eyre::eyre!("atomic replace: {e}"))?;

        message!(
            "✅ Reset complete. Restored snapshot + applied {} changeset(s).",
            cs_count
        );

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// snapshot
// ---------------------------------------------------------------------------

/// Create a new baseline snapshot
///
/// Forces creation of a new baseline snapshot, even if the schema hasn't
/// changed. Useful when many changesets have accumulated and replay time
/// is growing. Uploads the snapshot and resets the manifest's changeset list.
#[derive(Debug, Args)]
#[command(after_long_help = SNAPSHOT_AFTER_LONG_HELP)]
pub struct Snapshot {
    /// Optional message describing this snapshot
    #[arg(short, long)]
    message: Option<String>,

    /// Force snapshot even when local database is not at manifest head
    #[arg(long)]
    force: bool,
}

pub static SNAPSHOT_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Create a new baseline snapshot</dim>
  <b>stencila db snapshot</b>

  <dim># Create with a description</dim>
  <b>stencila db snapshot -m \"compact after batch processing\"</b>
"
);

impl Snapshot {
    pub async fn run(self) -> Result<()> {
        let (stencila_dir, db_path) = resolve_paths().await?;
        let ws_id = workspace_id()?;

        if !db_path.exists() {
            eyre::bail!(
                "No database at `{}`; nothing to snapshot",
                db_path.display()
            );
        }

        let manifest_path = stencila_dir.join(sync::MANIFEST_FILE);

        // Guard: snapshot is a compaction operation that replaces the manifest.
        // If the local DB is behind or diverged, snapshotting would silently
        // discard changeset history. Require up-to-date state unless --force.
        let s = sync::status(&stencila_dir, &db_path)?;
        if s.manifest_exists && !self.force {
            if s.diverged {
                eyre::bail!(
                    "Local database has diverged from the manifest. Run `stencila db reset` first, \
                     or use `stencila db snapshot --force` to snapshot the current local state anyway."
                );
            }
            if !s.up_to_date {
                eyre::bail!(
                    "Local database is behind the manifest ({}/{} changesets applied). \
                     Run `stencila db pull` first, or use `stencila db snapshot --force` to \
                     snapshot the current local state anyway.",
                    s.applied_changesets,
                    s.total_changesets
                );
            }
        }

        // Read local schema version
        let conn = rusqlite::Connection::open(&db_path)?;
        sync::ensure_sync_table(&conn)?;
        let local_schema = sync::schema_versions(&conn)?;
        drop(conn);

        let spinner = new_spinner("Compressing snapshot...");
        let compressed = sync::create_snapshot(&db_path)?;
        let hash = sync::sha256_hex(&compressed);
        let size = compressed.len() as u64;
        spinner.finish_and_clear();

        // Cache before uploading
        sync::write_cached_blob(&stencila_dir, "snapshots", &hash, &compressed)?;

        // Upload blob before writing manifest (invariant §5)
        let pb = new_bytes_bar(size, "Uploading snapshot");
        let pb_ref = pb.clone();
        stencila_cloud::db::upload_blob_with_progress(
            &ws_id,
            "snapshots",
            &hash,
            compressed,
            Some(Arc::new(move |sent| pb_ref.set_position(sent))),
        )
        .await?;
        pb.finish_and_clear();

        let manifest =
            sync::new_snapshot_manifest(hash.clone(), size, local_schema, self.message.clone());
        sync::write_manifest(&manifest_path, &manifest)?;

        let conn = rusqlite::Connection::open(&db_path)?;
        sync::set_sync_position(&conn, &hash)?;

        message!(
            "📦 Created snapshot {:.8} ({}). Changeset history reset. Remember to `git add .stencila/db.json`",
            hash,
            format_bytes(size)
        );

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// clean (local blob cache GC)
// ---------------------------------------------------------------------------

/// Clean up the local blob cache
///
/// Removes cached blobs that are no longer referenced by the current
/// manifest. Useful for reclaiming disk space after snapshot rotations.
#[derive(Debug, Args)]
#[command(after_long_help = CLEAN_AFTER_LONG_HELP)]
pub struct Clean {}

pub static CLEAN_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Remove unreferenced cached blobs</dim>
  <b>stencila db clean</b>
"
);

impl Clean {
    pub async fn run(self) -> Result<()> {
        let (stencila_dir, _db_path) = resolve_paths().await?;

        let manifest_path = stencila_dir.join(sync::MANIFEST_FILE);
        let manifest = sync::read_manifest(&manifest_path)?.ok_or_else(|| {
            eyre::eyre!(
                "No manifest at `{}`; nothing to clean against",
                manifest_path.display()
            )
        })?;

        let (removed, freed) = sync::clean_blob_cache(&stencila_dir, &manifest)?;

        if removed == 0 {
            message!("✅ Cache is clean — no orphaned blobs");
        } else {
            message!(
                "🧹 Removed {} cached blob(s), freed {}",
                removed,
                format_bytes(freed)
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// gc (remote blob garbage collection)
// ---------------------------------------------------------------------------

/// Remove orphaned remote blobs
///
/// Lists all blobs stored on Stencila Cloud for this workspace and removes
/// any that are not referenced by the current manifest. This cleans up blobs
/// left behind by rebased or force-pushed manifests.
#[derive(Debug, Args)]
#[command(after_long_help = GC_AFTER_LONG_HELP)]
pub struct Gc {
    /// Show what would be removed without actually deleting
    #[arg(long)]
    dry_run: bool,

    /// Run `git fetch --all` before scanning refs (without prompting)
    #[arg(long, conflicts_with = "no_fetch")]
    fetch: bool,

    /// Skip `git fetch` (without prompting)
    #[arg(long, conflicts_with = "fetch")]
    no_fetch: bool,
}

pub static GC_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Remove orphaned remote blobs</dim>
  <b>stencila db gc</b>

  <dim># Preview what would be removed</dim>
  <b>stencila db gc --dry-run</b>

  <dim># Non-interactive: fetch first, then GC</dim>
  <b>stencila db gc --fetch</b>

  <dim># Non-interactive: skip fetch</dim>
  <b>stencila db gc --no-fetch</b>
"
);

impl Gc {
    pub async fn run(self) -> Result<()> {
        let (stencila_dir, _db_path) = resolve_paths().await?;
        let ws_id = workspace_id()?;

        // Collect referenced hashes from ALL git refs, not just the
        // current checkout, to avoid deleting blobs needed by other branches.
        let referenced = self.collect_all_referenced_hashes(&stencila_dir).await?;

        if referenced.is_empty() {
            eyre::bail!("No manifests found in any git ref; cannot determine referenced blobs");
        }

        let spinner = new_spinner("Listing remote blobs...");

        let mut orphaned = Vec::new();
        for kind in &["snapshots", "changesets"] {
            let remote_hashes = stencila_cloud::db::list_blobs(&ws_id, kind).await?;
            for hash in remote_hashes {
                if !referenced.contains(&hash) {
                    orphaned.push((kind.to_string(), hash));
                }
            }
        }
        spinner.finish_and_clear();

        if orphaned.is_empty() {
            message!("✅ No orphaned remote blobs");
            return Ok(());
        }

        if self.dry_run {
            message!("Would remove {} orphaned blob(s):", orphaned.len());
            for (kind, hash) in &orphaned {
                let short = &hash[..hash.len().min(8)];
                message!("  {}/{}", kind, short);
            }
            return Ok(());
        }

        let pb = new_items_bar(orphaned.len() as u64, "blobs");
        for (kind, hash) in &orphaned {
            stencila_cloud::db::delete_blob(&ws_id, kind, hash).await?;
            pb.inc(1);
        }
        pb.finish_and_clear();

        message!("🧹 Removed {} orphaned remote blob(s)", orphaned.len());

        Ok(())
    }

    /// Collect all blob hashes referenced by db.json manifests across
    /// every git ref (branches, remote-tracking branches, tags).
    ///
    /// Offers to run `git fetch` first so remote-tracking refs are
    /// up-to-date, ensuring blobs used by collaborators' branches are
    /// not mistakenly treated as orphaned.
    async fn collect_all_referenced_hashes(
        &self,
        stencila_dir: &std::path::Path,
    ) -> Result<std::collections::HashSet<String>> {
        use stencila_codec_utils::{closest_git_repo, git_list_refs, git_show_file_at_ref};

        let repo_root = closest_git_repo(stencila_dir)?;

        // Compute the repo-relative path to .stencila/db.json
        let manifest_abs = stencila_dir.join(sync::MANIFEST_FILE);
        let manifest_rel = manifest_abs
            .strip_prefix(&repo_root)
            .unwrap_or(&manifest_abs);
        let manifest_rel_str = manifest_rel.to_string_lossy();

        // Decide whether to fetch: explicit flags take precedence,
        // otherwise prompt interactively.
        let do_fetch = if self.fetch {
            true
        } else if self.no_fetch {
            false
        } else {
            ask_with(
                "Run `git fetch --all` first to ensure remote-tracking refs are up-to-date?",
                AskOptions {
                    default: Some(Answer::Yes),
                    ..Default::default()
                },
            )
            .await?
            .is_yes()
        };

        if do_fetch {
            let spinner = new_spinner("Running git fetch --all...");
            let status = std::process::Command::new("git")
                .arg("-C")
                .arg(&repo_root)
                .args(["fetch", "--all", "--quiet"])
                .status();
            spinner.finish_and_clear();
            match status {
                Ok(status) if status.success() => {}
                Ok(status) => {
                    message!(
                        "⚠️  git fetch exited with {}; continuing with existing refs",
                        status
                    );
                }
                Err(error) => {
                    message!(
                        "⚠️  git fetch failed: {}; continuing with existing refs",
                        error
                    );
                }
            }
        }

        let spinner = new_spinner("Scanning git refs for manifests...");

        let refs = git_list_refs(&repo_root)?;
        let mut referenced = std::collections::HashSet::new();
        let mut manifests_found = 0usize;

        for ref_name in &refs {
            if let Some(content) = git_show_file_at_ref(&repo_root, ref_name, &manifest_rel_str)
                && let Ok(manifest) = serde_json::from_str::<sync::Manifest>(&content)
                && manifest.format == sync::MANIFEST_FORMAT
            {
                referenced.insert(manifest.base_snapshot.hash.clone());
                for cs in &manifest.changesets {
                    referenced.insert(cs.hash.clone());
                }
                manifests_found += 1;
            }
        }

        // Also include the current working-tree manifest (may contain
        // unpushed-to-git changes from a recent `db push`)
        let current_manifest_path = stencila_dir.join(sync::MANIFEST_FILE);
        if let Ok(Some(manifest)) = sync::read_manifest(&current_manifest_path) {
            referenced.insert(manifest.base_snapshot.hash.clone());
            for cs in &manifest.changesets {
                referenced.insert(cs.hash.clone());
            }
            manifests_found += 1;
        }

        spinner.finish_and_clear();

        message!(
            "Found {} manifest(s) across {} ref(s), {} referenced blob(s)",
            manifests_found,
            refs.len(),
            referenced.len()
        );

        Ok(referenced)
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

fn format_schema_version(sv: &std::collections::BTreeMap<String, i32>) -> String {
    if sv.is_empty() {
        return "(none)".to_string();
    }
    sv.iter()
        .map(|(d, v)| format!("{d}@{v}"))
        .collect::<Vec<_>>()
        .join(", ")
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
