use std::{
    collections::{BTreeMap, btree_map::Entry},
    env::current_dir,
    fs::read_dir,
    path::{Path, PathBuf},
    str::FromStr,
    time::UNIX_EPOCH,
};

use chrono::Utc;
use eyre::{OptionExt, Result, bail};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::Display;
use tokio::{
    self,
    fs::{read_to_string, remove_dir_all, remove_file, rename, write},
};
use url::Url;

use stencila_codecs::{DecodeOptions, EncodeOptions};
use stencila_dirs::{
    DB_FILE, DOCS_FILE, STORE_DIR, closest_stencila_dir, stencila_artifacts_dir, stencila_db_file,
    stencila_docs_file, stencila_store_dir, workspace_dir, workspace_relative_path,
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

    /// The last time the document was stored in the `store` directory
    pub stored_at: Option<u64>,

    /// The last time the document was added to the workspace database
    pub added_at: Option<u64>,

    /// The remotes that are tracked for the path
    pub remotes: Option<BTreeMap<Url, DocumentRemote>>,
}

impl Default for DocumentTracking {
    fn default() -> Self {
        Self {
            id: new_id(),
            stored_at: Default::default(),
            added_at: Default::default(),
            remotes: Default::default(),
        }
    }
}

impl DocumentTracking {
    pub fn store_file(&self) -> String {
        format!("{}.json", self.id)
    }

    pub fn store_path(&self, stencila_dir: &Path) -> PathBuf {
        stencila_dir.join(STORE_DIR).join(self.store_file())
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

        let status = if modified_at >= self.stored_at.map(|stored_at| stored_at.saturating_add(10))
        {
            DocumentTrackingStatus::Ahead
        } else if modified_at < self.stored_at {
            DocumentTrackingStatus::Behind
        } else {
            DocumentTrackingStatus::Synced
        };

        (status, modified_at)
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
}

#[derive(Default, Display, Serialize, Deserialize)]
pub enum DocumentTrackingStatus {
    /// The workspace file is of a format that does not support tracking
    #[default]
    Unsupported,

    /// There is an entry for the workspace file in the tracking file
    /// but that path no longer exists in the workspace directory
    Deleted,

    /// The workspace file is ahead of the tracking file: it has changed
    /// since it was last synced
    Ahead,

    /// The workspace file is behind the tracking file: there have been
    /// changes to the tracking file which have not been propagated
    /// to the workspace file
    Behind,

    /// The workspace file is synced with the tracking file: they have the
    /// same modification time
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
        let (_, _, store_path, ..) = Document::track_path(path, Some(time_now()), None).await?;

        let root = self.root.read().await;

        // Write the root node to storage
        stencila_codec_json::to_path(
            &root,
            &store_path,
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
    /// path to `.stencila/store/<ID>.json` and adding an entry
    /// to the `.stencila/docs.json` file.
    ///
    /// If the path is already being tracked (i.e. it has an entry
    /// in the `docs.json` file) returns true, otherwise false.
    #[tracing::instrument]
    pub async fn track_path(
        path: &Path,
        stored_at: Option<u64>,
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

        let store_dir = stencila_dir.join(STORE_DIR);
        let db_path = stencila_dir.join(DB_FILE);
        let relative_path = workspace_relative_path(&stencila_dir, path, true)?;

        match entries.entry(relative_path) {
            Entry::Occupied(mut occupied_entry) => {
                // Update existing entry
                let entry = occupied_entry.get_mut();
                let id = entry.id.clone();
                let store_path = store_dir.join(entry.store_file());

                entry.stored_at = stored_at;
                entry.added_at = added_at;

                write_entries(&stencila_dir, &entries).await?;

                Ok((id, true, store_path, db_path))
            }
            Entry::Vacant(vacant_entry) => {
                // Create a new entry
                let id = new_id();

                let entry = DocumentTracking {
                    id: id.clone(),
                    stored_at,
                    added_at,
                    ..Default::default()
                };
                let store_path = store_dir.join(entry.store_file());

                vacant_entry.insert(entry);
                write_entries(&stencila_dir, &entries).await?;

                Ok((id, false, store_path, db_path))
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
            let (doc_id, store_path, mut root) = if path.exists() {
                let (doc_id, _, store_path, _) =
                    Document::track_path(&path, Some(time_now()), Some(time_now())).await?;
                let root = Document::open(&path, decode_options.clone())
                    .await?
                    .root()
                    .await;
                (doc_id, store_path, root)
            } else {
                let doc_id = new_id();
                let store_dir = stencila_store_dir(stencila_dir, true).await?;
                let store_path = store_dir.join(format!("{doc_id}.json"));
                let root =
                    stencila_codecs::from_identifier(identifier, decode_options.clone()).await?;
                (doc_id, store_path, root)
            };

            if should_canonicalize {
                canonicalize(&mut root).await?;
            }

            // Store root node
            stencila_codec_json::to_path(
                &root,
                &store_path,
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
    /// deletes the corresponding `.stencila/store/<ID>.json`,
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
        let store_path = entry.store_path(&stencila_dir);
        if store_path.exists() {
            remove_file(store_path).await?;
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
        let store_dir = stencila_store_dir(&stencila_dir, false).await?;
        let entries = read_entries(&stencila_dir).await?;
        for path in read_dir(store_dir)?.flatten() {
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

        let store_dir = stencila_dir.join(STORE_DIR);
        let relative_path = workspace_relative_path(&stencila_dir, path, false)?;

        Ok(Some((store_dir, entries.remove(&relative_path))))
    }

    /// Get the tracking storage file path for a document path
    pub async fn tracking_storage(path: &Path) -> Result<Option<PathBuf>> {
        Ok(Document::tracking_path(path)
            .await?
            .and_then(|(store_dir, entry)| entry.map(|entry| store_dir.join(entry.store_file()))))
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

        let store_dir = stencila_store_dir(&stencila_dir, false).await?;
        if store_dir.exists() {
            remove_dir_all(&store_dir).await?;
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
