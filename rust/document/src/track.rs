use std::{
    collections::BTreeMap,
    env::current_dir,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use node_db::NodeDatabase;
use schema::{Node, NodeId};
use url::Url;

use codecs::EncodeOptions;
use common::{
    chrono::Utc,
    eyre::{bail, OptionExt, Result},
    itertools::Itertools,
    serde::{Deserialize, Serialize},
    serde_json,
    serde_with::skip_serializing_none,
    strum::Display,
    tokio::{
        self,
        fs::{create_dir_all, read_to_string, remove_file, rename, write},
    },
    tracing,
};
use format::Format;

use crate::{
    dirs::{closest_workspace_dir, workspace_dir, STENCILA_DIR},
    Document,
};

const TRACKING_FILE: &str = "docs.json";
const STORE_DIR: &str = "store";
const DB_DIR: &str = "db";

/// Get the path of the Stencila tracking file for a workspace directory
pub async fn tracking_file(workspace_dir: &Path, ensure: bool) -> Result<PathBuf> {
    let tracking_file = workspace_dir.join(STENCILA_DIR).join(TRACKING_FILE);

    if ensure && !tracking_file.exists() {
        write(&tracking_file, "{}\n").await?;
    }

    Ok(tracking_file)
}

/// Get the path of the database directory for a workspace directory
pub async fn db_dir(workspace_dir: &Path, ensure: bool) -> Result<PathBuf> {
    let db_dir = workspace_dir.join(STENCILA_DIR).join(DB_DIR);

    if ensure && !db_dir.exists() {
        create_dir_all(&db_dir).await?;
        node_db::NodeDatabase::new(&db_dir.to_string_lossy())?;
    }

    Ok(db_dir)
}

/// Get the path of closest tracking file to a path
async fn closest_tracking_file(path: &Path, ensure: bool) -> Result<PathBuf> {
    tracking_file(&closest_workspace_dir(path, ensure).await?, ensure).await
}

/// Get the closest tracking entries to a path, if any
async fn closest_tracking_entries(path: &Path) -> Result<Option<DocumentTrackingEntries>> {
    let tracking_file = closest_tracking_file(path, false).await?;
    if !tracking_file.exists() {
        return Ok(None);
    }

    let tracking_data = read_to_string(&tracking_file).await?;

    Ok(Some(serde_json::from_str(&tracking_data)?))
}

/// Read the closest `.stencila` dir and entries to a path and ensure they exist
///
/// If there is no closest tracking dir, one will be created.
async fn read_tracking(
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

    let tracking_file = closest_tracking_file(&origin_path, ensure).await?;
    if !tracking_file.exists() {
        return Ok(None);
    }
    let stencila_dir = tracking_file
        .parent()
        .ok_or_eyre("no parent")?
        .to_path_buf();

    let tracking_data = read_to_string(&tracking_file).await?;
    let entries = serde_json::from_str(&tracking_data).unwrap_or_default();

    Ok(Some((stencila_dir, entries)))
}

/// Write tracking entries to the tracking file in a tracking directory
async fn write_tracking(
    stencila_dir: &Path,
    tracking_entries: &DocumentTrackingEntries,
) -> Result<()> {
    let tracking_file = stencila_dir.join(TRACKING_FILE);
    let json = serde_json::to_string_pretty(tracking_entries)?;
    write(&tracking_file, json).await?;

    Ok(())
}

/// Make a path relative to the workspace directory of a tracking directory
fn workspace_relative_path(
    stencila_dir: &Path,
    doc_path: &Path,
    must_exist: bool,
) -> Result<PathBuf> {
    let workspace_dir = workspace_dir(stencila_dir)?.canonicalize()?;

    let relative_path = match doc_path.canonicalize() {
        // The document exists so make relative to the working directory
        Ok(doc_path) => match doc_path.strip_prefix(workspace_dir) {
            Ok(path) => path.to_path_buf(),
            Err(..) => bail!(
                "Path is not in the workspace being tracked: {}",
                doc_path.display()
            ),
        },
        // The document does not exist
        Err(..) => {
            if must_exist {
                bail!("File does not exist: {}", doc_path.display())
            }
            doc_path.to_path_buf()
        }
    };

    Ok(relative_path)
}

/// Create a new document id
fn new_id() -> NodeId {
    const NICK: [u8; 3] = [b'd', b'o', b'c'];
    NodeId::random(NICK)
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
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DocumentTracking {
    /// The tracking id for the document
    pub id: NodeId,

    /// The format of the file
    ///
    /// Mainly used so that the stored file can end in `.<EXT>.json`
    /// so that the user can use `.gitignore` patterns based on the extension.
    pub format: Format,

    /// The last time the document was stored in the tracking directory
    pub stored_at: Option<u64>,

    /// The source that the file was stored from
    pub stored_from: Option<String>,

    /// The remotes that are tracked for the path
    pub remotes: Option<BTreeMap<Url, DocumentRemote>>,
}

impl Default for DocumentTracking {
    fn default() -> Self {
        Self {
            id: new_id(),
            format: Default::default(),
            stored_at: Default::default(),
            stored_from: Default::default(),
            remotes: Default::default(),
        }
    }
}

impl DocumentTracking {
    pub fn store_file(&self) -> String {
        format!("{}.json", self.id)
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
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DocumentRemote {
    /// The last time the document was pulled from the remote
    pub pulled_at: Option<u64>,

    /// The last time the document was pushed from the remote
    pub pushed_at: Option<u64>,
}

#[derive(Default, Display, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
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
            Document::track_remote(path, remote).await?;
        } else {
            Document::track_path(path, None).await?;
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
    pub async fn store(&self) -> Result<()> {
        let Some(path) = &self.path else {
            bail!("Can't store document, it has no path yet; save it first")
        };

        // Get the storage path for the document, ensuring it is tracked
        let (doc_id, .., store_path, db_path) =
            Document::track_path(path, Some(time_now())).await?;

        let root = self.root.read().await;

        // Write the root node to storage
        codec_json::to_path(
            &root,
            &store_path,
            Some(EncodeOptions {
                compact: Some(false),
                ..Default::default()
            }),
        )?;

        // Update the document in the database
        let mut db = NodeDatabase::at(&db_path)?;
        db.upsert(&doc_id, &root)?;

        Ok(())
    }

    /// Restore a document from the workspace's `.stencila` directory
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

        Ok(codec_json::from_path(&storage_path, None)?.0)
    }

    /// Start tracking a document path
    ///
    /// Starts tracking the path by saving the document at the
    /// path to `.stencila/store/<ID>.json` and adding an entry
    /// to the tracking file at `.stencila/docs.json`.
    ///
    /// If the path is already being tracked (i.e. it has an entry
    /// in the tracking file) returns true, otherwise false.
    #[tracing::instrument]
    pub async fn track_path(
        path: &Path,
        stored_at: Option<u64>,
    ) -> Result<(NodeId, bool, PathBuf, PathBuf)> {
        if !(path.exists() && path.is_file()) {
            bail!("Path does not exist or is not a file: {}", path.display())
        }

        if !codecs::from_path_is_supported(path) {
            bail!("File format is not supported: {}", path.display())
        }

        let (stencila_dir, mut entries) = read_tracking(path, true)
            .await?
            .ok_or_eyre("no tracking file despite ensure")?;

        let store_dir = stencila_dir.join(STORE_DIR);
        let db_path = stencila_dir.join(DB_DIR);
        let relative_path = workspace_relative_path(&stencila_dir, path, true)?;

        // Reuse existing id or create a new one
        if entries.contains_key(&relative_path) {
            let entry = entries.get_mut(&relative_path).expect("checked above");
            let id = entry.id.clone();
            let store_path = store_dir.join(entry.store_file());

            if stored_at.is_some() {
                entry.stored_at = stored_at;
            }
            write_tracking(&stencila_dir, &entries).await?;

            Ok((id, true, store_path, db_path))
        } else {
            let id = new_id();
            let format = Format::from_path(path);

            let entry = DocumentTracking {
                id: id.clone(),
                format,
                stored_at,
                ..Default::default()
            };
            let store_path = store_dir.join(entry.store_file());

            entries.insert(relative_path, entry);
            write_tracking(&stencila_dir, &entries).await?;

            Ok((id, false, store_path, db_path))
        }
    }

    /// Start tracking a document remote
    ///
    /// Starts tracking the remote by saving the document at the path
    /// to `.stencila/track/<ID>.json` and, if necessary, adding an
    /// entry to the tracking file at `.stencila/track/docs.json`.
    ///
    /// If the path or remote is already being tracked (i.e. there is
    /// a corresponding entry in the tracking file) the exiting
    /// document id will be used. Returns true if both the path and the
    /// remote are being tracked, otherwise false.
    #[tracing::instrument]
    pub async fn track_remote(path: &Path, (url, remote): (Url, DocumentRemote)) -> Result<bool> {
        if !(path.exists() && path.is_file()) {
            bail!("Path does not exist or is not a file: {}", path.display())
        }

        if !codecs::from_path_is_supported(path) {
            bail!("File format is not supported: {}", path.display())
        }

        let (stencila_dir, mut entries) = read_tracking(path, true)
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

        let format = Format::from_path(path);

        // Update tracking file
        entries
            .entry(relative_path)
            .and_modify(|entry| {
                entry.format = format.clone();
                entry.remotes = remotes.clone();
            })
            .or_insert_with(|| DocumentTracking {
                id,
                format,
                remotes,
                ..Default::default()
            });
        write_tracking(&stencila_dir, &entries).await?;

        Ok(already_tracked)
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
    /// Removes the entry for the path in the tracking file at `.stencila/track/docs.json` and
    /// the corresponding `.stencila/track/<ID>.json`. Does not remove any other
    /// entries for the document id in the tracking file.
    ///
    /// Gives a warning if the path is not being tracked.
    #[tracing::instrument]
    pub async fn untrack_path(path: &Path) -> Result<()> {
        let Some((stencila_dir, mut entries)) = read_tracking(path, false).await? else {
            tracing::warn!("Path is not being tracked: {}", path.display());
            return Ok(());
        };
        let relative_path = workspace_relative_path(&stencila_dir, path, false)?;

        let mut storage_file = None;
        entries.retain(|path, entry| {
            if path == &relative_path {
                storage_file = Some(entry.store_file());
                false
            } else {
                true
            }
        });

        if let Some(storage_file) = storage_file {
            let stored_path = stencila_dir.join(storage_file);
            if stored_path.exists() {
                remove_file(stored_path).await?;
            }

            // Update tracking file
            write_tracking(&stencila_dir, &entries).await?;
        } else {
            tracing::warn!("Path is not being tracked: {}", path.display());
        }

        Ok(())
    }

    /// Stop tracking a document remote
    ///
    /// Removes the entry for the remote in the tracking file at `.stencila/track/docs.json`
    /// but does not remove the corresponding `.stencila/track/<ID>.json`.
    ///
    /// Gives a warning if the path, or the remote, is not being tracked.
    #[tracing::instrument]
    pub async fn untrack_remote(path: &Path, remote: &Url) -> Result<()> {
        let Some((stencila_dir, mut entries)) = read_tracking(path, false).await? else {
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
        write_tracking(&stencila_dir, &entries).await?;

        Ok(())
    }

    /// Move a tracked document
    ///
    /// Moves (renames) the file and updates the entry in the tracking file
    /// at `.stencila/track/docs.json`.
    ///
    /// If there is no entry for the `from` path then the `to` path will
    /// be tracked.
    ///
    /// Will error if the from `path` does not yet exist. Will create parent
    /// directories of the `to` path if necessary.
    #[tracing::instrument]
    pub async fn move_path(from: &Path, to: &Path) -> Result<()> {
        // This is a simple, unoptimized implementation
        Document::untrack_path(from).await?;
        rename(from, to).await?;
        Document::track_path(to, None).await?;

        Ok(())
    }

    /// Remove a tracked document
    ///
    /// Removes the file from the workspace, the corresponding
    /// entry in the tracking file at `.stencila/track/docs.json`,
    /// and the corresponding `.stencila/track/<ID>.json`.
    #[tracing::instrument]
    pub async fn remove_path(path: &Path) -> Result<()> {
        Document::untrack_path(path).await?;

        remove_file(path).await?;

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
        let Some((stencila_dir, mut entries)) = read_tracking(path, false).await? else {
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
        closest_tracking_entries(path).await
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
