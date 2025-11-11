use std::{
    collections::{BTreeMap, HashSet, btree_map::Entry},
    env::current_dir,
    fs::read_dir,
    path::{Path, PathBuf},
    str::FromStr,
    time::UNIX_EPOCH,
};

use chrono::Utc;
use clap::ValueEnum;
use eyre::{OptionExt, Result, bail};
use futures::future::join_all;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::{Display, EnumString};
use tokio::{
    self,
    fs::{read_to_string, remove_dir_all, remove_file, rename, write},
};
use url::Url;

use stencila_codecs::{DecodeOptions, EncodeOptions, remotes::RemoteService};
use stencila_dirs::{
    CACHE_DIR, DB_FILE, DOCS_FILE, closest_stencila_dir, stencila_artifacts_dir,
    stencila_cache_dir, stencila_db_file, stencila_docs_file, workspace_dir,
    workspace_relative_path,
};
use stencila_node_canonicalize::canonicalize;
use stencila_node_db::NodeDatabase;
use stencila_schema::{Node, NodeId};

use crate::Document;

/// Find the closest `.stencila/docs.json` file to a path and read it's entries
///
/// If `ensure` is true and there is no closest `docs.json` file, one will be created.
async fn closest_entries(
    path: &Path,
    ensure: bool,
) -> Result<Option<(PathBuf, DocumentTrackingEntries)>> {
    // Do not require that file exists because user may be un-tracking a file that
    // has already been deleted. If it does not exist then use the current
    // directory to resolve the tracking directory
    let origin_path = if path.exists() {
        path.to_path_buf()
    } else {
        current_dir()?
    };

    let stencila_dir = closest_stencila_dir(&origin_path, ensure).await?;
    let docs_file = stencila_docs_file(&stencila_dir, ensure).await?;

    if !docs_file.exists() {
        return Ok(None);
    }

    let entries = read_entries(&stencila_dir).await?;

    Ok(Some((stencila_dir, entries)))
}

/// Read document tracking entries from the `docs.json` file in a `.stencila` directory
async fn read_entries(stencila_dir: &Path) -> Result<DocumentTrackingEntries> {
    let docs_file = stencila_dir.join(DOCS_FILE);

    let json = read_to_string(&docs_file).await?;
    let entries = serde_json::from_str(&json)?;

    Ok(entries)
}

/// Write document tracking entries to the `docs.json` file in a `.stencila` directory
async fn write_entries(stencila_dir: &Path, entries: &DocumentTrackingEntries) -> Result<()> {
    let docs_file = stencila_dir.join(DOCS_FILE);

    let json = serde_json::to_string_pretty(entries)?;
    write(&docs_file, json).await?;

    Ok(())
}

/// Remove watch_ids that no longer exist on Stencila Cloud
///
/// Takes a set of valid watch IDs and removes any watch_id from the tracking
/// entries that is not in the provided set. Returns a list of removed watches
/// for display purposes.
pub async fn remove_deleted_watches(
    path: &Path,
    valid_watch_ids: &HashSet<u64>,
) -> Result<Vec<(PathBuf, Url, u64)>> {
    let mut deleted_watches = Vec::new();

    let Some((stencila_dir, mut entries)) = closest_entries(path, false).await? else {
        return Ok(deleted_watches);
    };

    let mut write_needed = false;

    for (path, tracking) in entries.iter_mut() {
        if let Some(remotes) = &mut tracking.remotes {
            for (remote_url, remote) in remotes.iter_mut() {
                if let Some(watch_id_str) = &remote.watch_id
                    && let Ok(watch_id) = watch_id_str.parse::<u64>()
                    && !valid_watch_ids.contains(&watch_id)
                {
                    // Watch no longer exists, remove it
                    remote.watch_id = None;
                    remote.watch_direction = None;
                    write_needed = true;

                    deleted_watches.push((path.clone(), remote_url.clone(), watch_id));
                }
            }
        }
    }

    // Write back to docs.json if any watch_ids were removed
    if write_needed {
        write_entries(&stencila_dir, &entries).await?;
    }

    Ok(deleted_watches)
}

/// Create a new document id
fn new_id() -> NodeId {
    NodeId::random(*b"doc")
}

/// Get the timestamp now
fn time_now() -> u64 {
    Utc::now().timestamp() as u64
}

/// Get the modification time of a path
fn time_modified(path: &Path) -> Result<u64> {
    let metadata = std::fs::File::open(path)?.metadata()?;
    Ok(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs())
}

pub type DocumentTrackingEntries = BTreeMap<PathBuf, DocumentTracking>;
pub type DocumentRemoteEntries = BTreeMap<Url, DocumentRemote>;

/// Tracking information for a tracked location
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentTracking {
    /// The tracking id for the document
    pub id: NodeId,

    /// The last time the document was cached
    pub cached_at: Option<u64>,

    /// The last time the document was added to the workspace database
    pub added_at: Option<u64>,

    /// The remotes that are tracked for the path
    pub remotes: Option<BTreeMap<Url, DocumentRemote>>,
}

impl Default for DocumentTracking {
    fn default() -> Self {
        Self {
            id: new_id(),
            cached_at: Default::default(),
            added_at: Default::default(),
            remotes: Default::default(),
        }
    }
}

impl DocumentTracking {
    pub fn store_file(&self) -> String {
        format!("{}.json", self.id)
    }

    pub fn cache_path(&self, stencila_dir: &Path) -> PathBuf {
        stencila_dir.join(CACHE_DIR).join(self.store_file())
    }

    pub fn status(
        &self,
        workspace_dir: &Path,
        path: &Path,
    ) -> (DocumentTrackingStatus, Option<u64>) {
        let path = workspace_dir.join(path);

        if !path.exists() {
            return (DocumentTrackingStatus::Deleted, None);
        }

        let modified_at = time_modified(&path).ok();

        let status = if modified_at >= self.cached_at.map(|cached_at| cached_at.saturating_add(10))
        {
            DocumentTrackingStatus::Ahead
        } else if modified_at < self.cached_at {
            DocumentTrackingStatus::Behind
        } else {
            DocumentTrackingStatus::Synced
        };

        (status, modified_at)
    }

    /// Get the status and modification time for all tracked remotes
    ///
    /// Fetches metadata from each remote service in parallel and compares with local modification time,
    /// as well as pulled_at and pushed_at times to detect divergence.
    /// Returns a map of URL to (remote_modified_at, status).
    ///
    /// If fetching metadata fails for a remote, it will be included in the result with None for modified_at
    /// and Unknown status.
    pub async fn remote_statuses(
        &self,
        local_status: DocumentTrackingStatus,
        local_modified_at: Option<u64>,
    ) -> BTreeMap<Url, (Option<u64>, DocumentTrackingStatus)> {
        let Some(remotes) = &self.remotes else {
            return BTreeMap::new();
        };

        // Create futures to fetch metadata for each remote
        let futures = remotes.iter().map(|(url, remote)| async move {
            // Fetch metadata from remote service
            let remote_modified_at = async {
                match RemoteService::from_url(url) {
                    Some(RemoteService::GoogleDocs) => stencila_codec_gdoc::get_metadata(url).await,
                    Some(RemoteService::Microsoft365) => {
                        stencila_codec_m365::get_metadata(url).await
                    }
                    None => bail!("Unsupported remote service: {url}"),
                }
            }
            .await
            .ok();

            tracing::debug!("Remote {url} modified at {remote_modified_at:?}");

            /// Tolerance in seconds for local file modification time comparisons
            const LOCAL_TOLERANCE: u64 = 5;

            /// Tolerance in seconds for remote modification time comparisons
            /// (accounts for cloud service processing delays)
            const REMOTE_TOLERANCE: u64 = 30;

            // Calculate status by comparing local/remote modified times with pushed_at/pulled_at
            let status = if local_status == DocumentTrackingStatus::Deleted {
                DocumentTrackingStatus::Ahead
            } else {
                match (
                    local_modified_at,
                    remote_modified_at,
                    remote.pushed_at,
                    remote.pulled_at,
                ) {
                    (Some(local), Some(remote_mod), Some(pushed), Some(pulled)) => {
                        // Use the most recent sync time (push or pull) as our reference point
                        let last_synced = pushed.max(pulled);

                        // Check if local or remote have changed since last sync
                        let local_changed = local > last_synced.saturating_add(LOCAL_TOLERANCE);
                        let remote_changed =
                            remote_mod > last_synced.saturating_add(REMOTE_TOLERANCE);

                        if local_changed && remote_changed {
                            DocumentTrackingStatus::Diverged
                        } else if local_changed {
                            DocumentTrackingStatus::Behind
                        } else if remote_changed {
                            DocumentTrackingStatus::Ahead
                        } else {
                            DocumentTrackingStatus::Synced
                        }
                    }
                    (Some(local), Some(remote_mod), Some(pushed), None) => {
                        // Only pushed_at exists, use it as reference
                        let local_changed = local > pushed.saturating_add(LOCAL_TOLERANCE);
                        let remote_changed = remote_mod > pushed.saturating_add(REMOTE_TOLERANCE);

                        if local_changed && remote_changed {
                            DocumentTrackingStatus::Diverged
                        } else if local_changed {
                            DocumentTrackingStatus::Behind
                        } else if remote_changed {
                            DocumentTrackingStatus::Ahead
                        } else {
                            DocumentTrackingStatus::Synced
                        }
                    }
                    (Some(local), Some(remote_mod), None, Some(pulled)) => {
                        // Only pulled_at exists, use it as reference
                        let local_changed = local > pulled.saturating_add(LOCAL_TOLERANCE);
                        let remote_changed = remote_mod > pulled.saturating_add(REMOTE_TOLERANCE);

                        if local_changed && remote_changed {
                            DocumentTrackingStatus::Diverged
                        } else if local_changed {
                            DocumentTrackingStatus::Behind
                        } else if remote_changed {
                            DocumentTrackingStatus::Ahead
                        } else {
                            DocumentTrackingStatus::Synced
                        }
                    }
                    (Some(local), Some(remote_mod), _, _) => {
                        // Fallback: if we don't have pushed_at/pulled_at, just compare modified times
                        if local > remote_mod.saturating_add(REMOTE_TOLERANCE) {
                            DocumentTrackingStatus::Behind
                        } else if remote_mod > local.saturating_add(REMOTE_TOLERANCE) {
                            DocumentTrackingStatus::Ahead
                        } else {
                            DocumentTrackingStatus::Synced
                        }
                    }
                    _ => DocumentTrackingStatus::Unknown,
                }
            };

            (url.clone(), (remote_modified_at, status))
        });

        // Execute all futures in parallel and collect results into a BTreeMap
        join_all(futures).await.into_iter().collect()
    }
}

#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRemote {
    /// The last time the document was pulled from the remote
    pub pulled_at: Option<u64>,

    /// The last time the document was pushed from the remote
    pub pushed_at: Option<u64>,

    /// The watch ID from Stencila Cloud (if watch is enabled)
    pub watch_id: Option<String>,

    /// The watch direction (bi-directional, from-remote, or to-remote)
    pub watch_direction: Option<WatchDirection>,
}

/// The direction of synchronization for a watched remote
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum WatchDirection {
    /// Bi-directional sync: changes from remote create PRs, changes to repo push to remote
    #[default]
    Bi,

    /// One-way sync from remote: only remote changes create PRs
    FromRemote,

    /// One-way sync to remote: only repo changes push to remote
    ToRemote,
}

/// The pull request mode for a watched remote
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum WatchPrMode {
    /// Create PRs as drafts (default)
    #[default]
    Draft,

    /// Create PRs ready for review
    Ready,
}

impl DocumentRemote {
    /// Check if this remote is being watched
    pub fn is_watched(&self) -> bool {
        self.watch_id.is_some()
    }
}

#[derive(Debug, Default, Display, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentTrackingStatus {
    #[default]
    Unknown,

    /// There is an entry for the file but that path no longer exists in the
    /// workspace directory
    Deleted,

    /// Remote has changes that need to be pulled
    Ahead,

    /// Local file has changes that need to be pushed
    Behind,

    /// Both local and remote have changes since last sync
    Diverged,

    /// Local and remote are in sync
    Synced,
}

impl Document {
    /// Start, or continue, tracking the document
    ///
    /// Will error if the document does not have path yet (i.e. if
    /// it is new and has not been saved yet).
    ///
    /// See [`Document::track_path`].
    #[tracing::instrument(skip(self))]
    pub async fn track(&self, remote: Option<(Url, DocumentRemote)>) -> Result<()> {
        let Some(path) = &self.path else {
            bail!("Can't track document, it has no path yet; save it first")
        };

        if let Some(remote) = remote {
            Document::track_path_with_remote(path, remote).await?;
        } else {
            Document::track_path(path, None, None).await?;
        }

        Ok(())
    }

    /// Start, or continue, tracking the document with a record of it
    /// being pulled from a remote
    pub async fn track_remote_pulled(&self, remote: Url) -> Result<()> {
        let tracking = self
            .tracking()
            .await?
            .and_then(|(.., entry)| entry)
            .and_then(|entry| entry.remotes)
            .and_then(|mut remotes| remotes.remove(&remote))
            .unwrap_or_default();

        self.track(Some((
            remote,
            DocumentRemote {
                pulled_at: Some(time_now()),
                ..tracking
            },
        )))
        .await
    }

    /// Start, or continue, tracking the document with a record of it
    /// being pushed to a remote
    pub async fn track_remote_pushed(&self, remote: Url) -> Result<()> {
        let tracking = self
            .tracking()
            .await?
            .and_then(|(.., entry)| entry)
            .and_then(|entry| entry.remotes)
            .and_then(|mut remotes| remotes.remove(&remote))
            .unwrap_or_default();

        self.track(Some((
            remote,
            DocumentRemote {
                pushed_at: Some(time_now()),
                ..tracking
            },
        )))
        .await
    }

    /// Store the document in the workspace's `.stencila` directory
    #[tracing::instrument(skip(self))]
    pub async fn store(&self) -> Result<()> {
        let Some(path) = &self.path else {
            bail!("Can't store document, it has no path yet; save it first")
        };

        // Get the storage and database paths for the document, ensuring both exist
        let (_, _, cache_path, ..) = Document::track_path(path, Some(time_now()), None).await?;

        let root = self.root.read().await;

        // Write the root node to storage
        stencila_codec_json::to_path(
            &root,
            &cache_path,
            Some(EncodeOptions {
                compact: Some(false),
                ..Default::default()
            }),
        )?;

        Ok(())
    }

    /// Restore a document from the workspace's `.stencila` directory
    #[tracing::instrument]
    pub fn restore(path: &Path) -> Result<Node> {
        // Get the storage path for the document
        // Currently, this function needs to be sync because of where is is called from
        // and hence we need to do this blocking to await on `tracking_storage`.
        // This may be able to be avoided in the future.
        let result = tokio::task::block_in_place(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move { Document::tracking_storage(path).await })
        });
        let Some(storage_path) = result? else {
            bail!("No tracking storage path for document {}", path.display())
        };

        Ok(stencila_codec_json::from_path(&storage_path, None)?.0)
    }

    /// Start tracking a document path
    ///
    /// Starts tracking the path by saving the document at the
    /// path to `.stencila/cache/<ID>.json` and adding an entry
    /// to the `.stencila/docs.json` file.
    ///
    /// If the path is already being tracked (i.e. it has an entry
    /// in the `docs.json` file) returns true, otherwise false.
    #[tracing::instrument]
    pub async fn track_path(
        path: &Path,
        cached_at: Option<u64>,
        added_at: Option<u64>,
    ) -> Result<(NodeId, bool, PathBuf, PathBuf)> {
        if !(path.exists() && path.is_file()) {
            bail!("Path does not exist or is not a file: {}", path.display())
        }

        if !stencila_codecs::from_path_is_supported(path) {
            bail!("File format is not supported: {}", path.display())
        }

        let (stencila_dir, mut entries) = closest_entries(path, true)
            .await?
            .ok_or_eyre("no tracking file despite ensure")?;

        let cache_dir = stencila_dir.join(CACHE_DIR);
        let db_path = stencila_dir.join(DB_FILE);
        let relative_path = workspace_relative_path(&stencila_dir, path, true)?;

        match entries.entry(relative_path) {
            Entry::Occupied(mut occupied_entry) => {
                // Update existing entry
                let entry = occupied_entry.get_mut();
                let id = entry.id.clone();
                let cache_path = cache_dir.join(entry.store_file());

                entry.cached_at = cached_at;
                entry.added_at = added_at;

                write_entries(&stencila_dir, &entries).await?;

                Ok((id, true, cache_path, db_path))
            }
            Entry::Vacant(vacant_entry) => {
                // Create a new entry
                let id = new_id();

                let entry = DocumentTracking {
                    id: id.clone(),
                    cached_at,
                    added_at,
                    ..Default::default()
                };
                let cache_path = cache_dir.join(entry.store_file());

                vacant_entry.insert(entry);
                write_entries(&stencila_dir, &entries).await?;

                Ok((id, false, cache_path, db_path))
            }
        }
    }

    /// Start tracking a document remote
    ///
    /// Starts tracking the remote by saving the document at the path
    /// to `.stencila/track/<ID>.json` and, if necessary, adding an
    /// entry to the tracking file at `.stencila/docs.json`.
    ///
    /// If the path or remote is already being tracked (i.e. there is
    /// a corresponding entry in the tracking file) the exiting
    /// document id will be used. Returns true if both the path and the
    /// remote are being tracked, otherwise false.
    #[tracing::instrument]
    pub async fn track_path_with_remote(
        path: &Path,
        (url, remote): (Url, DocumentRemote),
    ) -> Result<bool> {
        if !(path.exists() && path.is_file()) {
            bail!("Path does not exist or is not a file: {}", path.display())
        }

        if !stencila_codecs::from_path_is_supported(path) {
            bail!("File format is not supported: {}", path.display())
        }

        let (stencila_dir, mut entries) = closest_entries(path, true)
            .await?
            .ok_or_eyre("no tracking file despite ensure")?;
        let relative_path = workspace_relative_path(&stencila_dir, path, true)?;

        // Reuse existing id or create a new one, update existing remotes or create new ones
        let (already_tracked, id, remotes) = match entries.get_mut(&relative_path) {
            Some(entry) => {
                let (already_tracked, remotes) = if let Some(remotes) = entry.remotes.as_mut() {
                    remotes.insert(url, remote);
                    (true, Some(remotes.clone()))
                } else {
                    (
                        false,
                        Some(DocumentRemoteEntries::from([(url.clone(), remote)])),
                    )
                };

                (already_tracked, entry.id.clone(), remotes)
            }
            None => (
                false,
                new_id(),
                Some(DocumentRemoteEntries::from([(url.clone(), remote)])),
            ),
        };

        // Update tracking file
        entries
            .entry(relative_path)
            .and_modify(|entry| {
                entry.remotes = remotes.clone();
            })
            .or_insert_with(|| DocumentTracking {
                id,
                remotes,
                ..Default::default()
            });
        write_entries(&stencila_dir, &entries).await?;

        Ok(already_tracked)
    }

    /// Add documents to a workspace database
    #[tracing::instrument(skip(identifiers))]
    pub async fn add_docs(
        stencila_dir: &Path,
        identifiers: &[String],
        decode_options: Option<DecodeOptions>,
        should_canonicalize: bool,
    ) -> Result<()> {
        let db_path = stencila_db_file(stencila_dir, true).await?;
        let mut db = NodeDatabase::new(&db_path)?;

        // Open each document, store it and upsert to database
        for identifier in identifiers {
            let path = PathBuf::from(identifier);
            let (doc_id, cache_path, mut root) = if path.exists() {
                let (doc_id, _, cache_path, _) =
                    Document::track_path(&path, Some(time_now()), Some(time_now())).await?;
                let root = Document::open(&path, decode_options.clone())
                    .await?
                    .root()
                    .await;
                (doc_id, cache_path, root)
            } else {
                let doc_id = new_id();
                let cache_dir = stencila_cache_dir(stencila_dir, true).await?;
                let cache_path = cache_dir.join(format!("{doc_id}.json"));
                let root =
                    stencila_codecs::from_identifier(identifier, decode_options.clone()).await?;
                (doc_id, cache_path, root)
            };

            if should_canonicalize {
                canonicalize(&mut root).await?;
            }

            // Store root node
            stencila_codec_json::to_path(
                &root,
                &cache_path,
                Some(EncodeOptions {
                    compact: Some(false),
                    ..Default::default()
                }),
            )?;

            // Upsert root node to database
            db.upsert(&doc_id, &root)?;
        }

        Ok(())
    }

    /// Remove documents from a workspace database
    #[tracing::instrument(skip(identifiers))]
    pub async fn remove_docs(stencila_dir: &Path, identifiers: &[String]) -> Result<()> {
        let db_path = stencila_db_file(stencila_dir, false).await?;
        if !db_path.exists() {
            return Ok(());
        }

        let mut db = NodeDatabase::new(&db_path)?;

        // Remove any db nodes for document
        let entries = read_entries(stencila_dir).await?;
        for identifier in identifiers {
            let path = PathBuf::from(identifier);
            let relative_path = workspace_relative_path(stencila_dir, &path, false)?;

            let Some(entry) = entries.get(&relative_path) else {
                continue;
            };

            // Delete database nodes
            db.delete(&entry.id)?;
        }

        Ok(())
    }

    /// Stop tracking the document
    ///
    /// Will error if the document does not have a path yet (i.e. if
    /// it is new and has not been saved yet).
    ///
    /// See [`Document::untrack_path`].
    #[tracing::instrument(skip(self))]
    pub async fn untrack(&self) -> Result<()> {
        let Some(path) = &self.path else {
            bail!("Can't untrack document, it has no path yet")
        };

        Document::untrack_path(path).await
    }

    /// Stop tracking a document path
    ///
    /// Removes the entry for the path in `.stencila/docs.json`,
    /// deletes the corresponding `.stencila/cache/<ID>.json`,
    /// and removes nodes for the document from the workspace database.
    ///
    /// Gives a warning if the path is not being tracked.
    #[tracing::instrument]
    pub async fn untrack_path(path: &Path) -> Result<()> {
        let Some((stencila_dir, mut entries)) = closest_entries(path, false).await? else {
            tracing::warn!("Path is not being tracked: {}", path.display());
            return Ok(());
        };
        let relative_path = workspace_relative_path(&stencila_dir, path, false)?;

        // Remove from entries
        let Some(entry) = entries.remove(&relative_path) else {
            tracing::warn!("Path is not being tracked: {}", path.display());
            return Ok(());
        };
        write_entries(&stencila_dir, &entries).await?;

        // Remove from database
        if entry.added_at.is_some() {
            let db_path = stencila_db_file(&stencila_dir, false).await?;
            if db_path.exists() {
                let mut db = NodeDatabase::new(&db_path)?;
                db.delete(&entry.id)?;
            }
        }

        // Remove store file
        let cache_path = entry.cache_path(&stencila_dir);
        if cache_path.exists() {
            remove_file(cache_path).await?;
        }

        Ok(())
    }

    /// Stop tracking a document remote
    ///
    /// Removes the entry for the remote in the tracking file at `.stencila/docs.json`
    /// but does not remove the corresponding `.stencila/track/<ID>.json`.
    ///
    /// Gives a warning if the path, or the remote, is not being tracked.
    #[tracing::instrument]
    pub async fn untrack_remote(path: &Path, remote: &Url) -> Result<()> {
        let Some((stencila_dir, mut entries)) = closest_entries(path, false).await? else {
            tracing::warn!("Path is not being tracked: {}", path.display());
            return Ok(());
        };
        let relative_path = workspace_relative_path(&stencila_dir, path, false)?;

        let Some(entry) = entries.get_mut(&relative_path) else {
            tracing::warn!("Path is not being tracked: {}", path.display());
            return Ok(());
        };

        // Remove the remote
        if entry
            .remotes
            .as_mut()
            .and_then(|remotes| remotes.remove(remote))
            .is_none()
        {
            tracing::warn!(
                "Remote is note being tracked for path `{}`: {}",
                path.display(),
                remote
            );
            return Ok(());
        }

        // Update tracking file
        write_entries(&stencila_dir, &entries).await?;

        Ok(())
    }

    /// Untrack all paths currently tracked files
    pub async fn untrack_all(dir: &Path) -> Result<()> {
        let statuses = match Document::tracking_all(dir).await? {
            Some(statuses) => statuses,
            None => {
                tracing::warn!("Current folder is not being tracked by Stencila");
                return Ok(());
            }
        };
        for (path, ..) in statuses {
            Document::untrack_path(&path).await?;
        }

        Ok(())
    }

    /// Move a tracked document
    ///
    /// Moves (renames) the file and updates the entry in the tracking file
    /// at `.stencila/docs.json`.
    ///
    /// If there is no entry for the `from` path then the `to` path will
    /// be tracked.
    ///
    /// Will create parent directories of the `to` path if necessary.
    #[tracing::instrument]
    pub async fn move_path(from: &Path, to: &Path) -> Result<()> {
        // Move the file if it exists
        if from.exists() {
            rename(from, to).await?;
        }

        // If the `from` path is already being tracked then just change the relative path for the entry.
        if let Some((tracking_dir, mut entries)) = closest_entries(from, false).await? {
            let from_relative_path = workspace_relative_path(&tracking_dir, from, false)?;
            if let Some(entry) = entries.remove(&from_relative_path) {
                let to_relative_path = workspace_relative_path(&tracking_dir, to, false)?;
                entries.insert(to_relative_path, entry);
                write_entries(&tracking_dir, &entries).await?;
                return Ok(());
            }
        }

        // Otherwise, if `from` is not being tracked already, then just track `to`
        Document::track_path(to, None, None).await?;

        Ok(())
    }

    /// Untrack all files that have been deleted, ensure no unneeded cache files, and remove all artifacts
    pub async fn clean(dir: &Path) -> Result<()> {
        let statuses = match Document::tracking_all(dir).await? {
            Some(statuses) => statuses,
            None => {
                tracing::warn!("Current folder is not being tracked by Stencila");
                return Ok(());
            }
        };

        let stencila_dir = closest_stencila_dir(dir, false).await?;
        let workspace_dir = workspace_dir(&stencila_dir)?;

        let answer = stencila_ask::ask(&format!("Are you sure you want to clean {}? Unused cache files, and all artifact directories, within it will be deleted.", stencila_dir.display())).await?;
        if answer.is_no() {
            return Ok(());
        }

        // Untrack all deleted paths
        for (path, tracking) in statuses {
            if let (DocumentTrackingStatus::Deleted, ..) = tracking.status(&workspace_dir, &path) {
                Document::untrack_path(&path).await?;
            }
        }

        // Remove all store files that do not have an entry for some reason
        let cache_dir = stencila_cache_dir(&stencila_dir, false).await?;
        let entries = read_entries(&stencila_dir).await?;
        for path in read_dir(cache_dir)?.flatten() {
            let path = path.path();
            let Some(id) = path
                .file_stem()
                .and_then(|id| id.to_str())
                .and_then(|id| NodeId::from_str(id).ok())
            else {
                continue;
            };

            if !entries.iter().any(|(.., entry)| entry.id == id) {
                remove_file(path).await?;
            }
        }

        // Remove all artifacts
        let artifacts_dir = stencila_artifacts_dir(&stencila_dir, false).await?;
        if artifacts_dir.exists() {
            remove_dir_all(artifacts_dir).await?;
        }

        Ok(())
    }

    /// Get the tracking information for the document
    ///
    /// Will error if the document does not have path yet (i.e. if
    /// it is new and has not been saved yet).
    ///
    /// See [`Document::tracking_path`].
    pub async fn tracking(&self) -> Result<Option<(PathBuf, Option<DocumentTracking>)>> {
        let Some(path) = &self.path else {
            return Ok(None);
        };

        Document::tracking_path(path).await
    }

    /// Is the document being tracked?
    pub async fn is_tracked(&self) -> bool {
        match self.tracking().await {
            Ok(info) => matches!(info, Some((.., Some(..)))),
            Err(error) => {
                tracing::error!("While tracking: {error}");
                false
            }
        }
    }

    /// Get the tracking information for a document path
    ///
    /// Returns the path of the store directory and the tracking info for the document if any.
    pub async fn tracking_path(path: &Path) -> Result<Option<(PathBuf, Option<DocumentTracking>)>> {
        let Some((stencila_dir, mut entries)) = closest_entries(path, false).await? else {
            return Ok(None);
        };

        let cache_dir = stencila_dir.join(CACHE_DIR);
        let relative_path = workspace_relative_path(&stencila_dir, path, false)?;

        Ok(Some((cache_dir, entries.remove(&relative_path))))
    }

    /// Get the tracking storage file path for a document path
    pub async fn tracking_storage(path: &Path) -> Result<Option<PathBuf>> {
        Ok(Document::tracking_path(path)
            .await?
            .and_then(|(cache_dir, entry)| entry.map(|entry| cache_dir.join(entry.store_file()))))
    }

    /// Get the tracking information for all tracked files in the workspace
    ///
    /// Finds the closest `.stencila` directory to the path (be it a file or directory),
    /// reads the tracking file, if any, and returns a vector tracking information
    pub async fn tracking_all(path: &Path) -> Result<Option<DocumentTrackingEntries>> {
        Ok(closest_entries(path, false)
            .await?
            .map(|(.., entries)| entries))
    }

    /// Rebuild the store and db directories
    ///
    /// Deletes any existing `store` and `db` directories and re-stores document's in
    /// the `docs.json file`.
    ///
    /// Useful if any changes to the Stencila Schema require a rebuild of the stored
    /// JSON and/or database without having to remove other tracking information.
    pub async fn tracking_rebuild(path: &Path) -> Result<()> {
        let Some((stencila_dir, entries)) = closest_entries(path, false).await? else {
            bail!("No `.stencila/docs.json` entries to rebuild")
        };

        let cache_dir = stencila_cache_dir(&stencila_dir, false).await?;
        if cache_dir.exists() {
            remove_dir_all(&cache_dir).await?;
        }

        let db_file = stencila_db_file(&stencila_dir, false).await?;
        if db_file.exists() {
            remove_file(&db_file).await?;
        }

        let identifiers = entries
            .into_keys()
            .map(|path| path.to_string_lossy().to_string())
            .collect_vec();
        Self::add_docs(&stencila_dir, &identifiers, None, true).await?;

        Ok(())
    }

    /// Returns a list of tracked remotes for a document
    ///
    /// Used when pushing or pulling the document. If the document is not tracked
    /// or has no tracked remotes, will return an empty vector.
    pub async fn remotes(&self) -> Result<Vec<Url>> {
        match self
            .tracking()
            .await?
            .and_then(|(.., entry)| entry)
            .and_then(|entry| entry.remotes)
            .map(|remotes| remotes.keys().cloned().collect_vec())
        {
            Some(urls) => Ok(urls),
            None => Ok(Vec::new()),
        }
    }
}
