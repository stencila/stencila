use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use fs4::tokio::AsyncFileExt;

use common::{
    eyre::{bail, OptionExt, Result},
    futures::future::try_join_all,
    itertools::Itertools,
    serde::Serialize,
    serde_with::skip_serializing_none,
    strum::Display,
    tokio::{
        fs::{create_dir_all, read_dir, read_to_string, remove_file, File, OpenOptions},
        io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom},
    },
    tracing,
};
use schema::{Node, NodeId};

use crate::Document;

#[derive(Default, Display, Serialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub enum DocumentStatusFlag {
    Deleted,
    #[default]
    Unsupported,
    Untracked,
    Unsaved,
    Ahead,
    Behind,
    Synced,
}

#[skip_serializing_none]
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DocumentStatus {
    pub path: Option<PathBuf>,
    pub status: DocumentStatusFlag,
    pub modified_at: Option<u64>,
    pub tracked_at: Option<u64>,
    pub doc_id: Option<String>,
}

impl DocumentStatus {
    fn unsaved() -> Self {
        Self {
            status: DocumentStatusFlag::Unsaved,
            ..Default::default()
        }
    }

    fn deleted(path: &Path) -> Self {
        Self {
            status: DocumentStatusFlag::Deleted,
            path: Some(path.to_path_buf()),
            ..Default::default()
        }
    }

    fn unsupported(path: &Path) -> Self {
        Self {
            status: DocumentStatusFlag::Unsupported,
            path: Some(path.to_path_buf()),
            ..Default::default()
        }
    }

    fn untracked(path: &Path, modified_at: u64) -> Self {
        Self {
            status: DocumentStatusFlag::Untracked,
            path: Some(path.to_path_buf()),
            modified_at: Some(modified_at),
            ..Default::default()
        }
    }

    fn new(path: &Path, modified_at: u64, tracked_at: u64, doc_id: String) -> Self {
        use DocumentStatusFlag::*;
        let status = match modified_at.cmp(&tracked_at) {
            Ordering::Equal => Synced,
            Ordering::Greater => Ahead,
            Ordering::Less => Behind,
        };

        Self {
            status,
            path: Some(path.to_path_buf()),
            modified_at: Some(modified_at),
            tracked_at: Some(tracked_at),
            doc_id: Some(doc_id),
            ..Default::default()
        }
    }
}

impl Document {
    /// Get the id of a document
    pub async fn id(&self) -> Option<String> {
        let root = &*self.root.read().await;

        match root {
            Node::Article(article) => article.id.clone(),
            Node::Chat(chat) => chat.id.clone(),
            Node::Prompt(prompt) => prompt.id.clone(),
            _ => None,
        }
    }

    /// Track a document
    ///
    /// Ensures that the root node of the document has an `id`
    /// and that `id` is not already being used in tracking directory.
    #[tracing::instrument(skip(self))]
    pub async fn track(&self) -> Result<()> {
        tracing::trace!("Tracking document");

        // Get the existing document id, or else generate one
        let id = self
            .mutate(|root| {
                let id = if let Node::Article(article) = root {
                    if let Some(id) = &article.id {
                        id
                    } else {
                        article.id = Some(id_random());
                        article.id.as_ref().expect("just assigned")
                    }
                } else if let Node::Chat(chat) = root {
                    if let Some(id) = &chat.id {
                        id
                    } else {
                        chat.id = Some(id_random());
                        chat.id.as_ref().expect("just assigned")
                    }
                } else if let Node::Prompt(prompt) = root {
                    if let Some(id) = &prompt.id {
                        id
                    } else {
                        prompt.id = Some(id_random());
                        prompt.id.as_ref().expect("just assigned")
                    }
                } else {
                    bail!(
                        "Tracking of `{}` documents is not yet supported",
                        root.node_type()
                    )
                };
                Ok(id.clone())
            })
            .await?;

        let Some(path) = &self.path else {
            bail!("Can't track document, it has no path yet")
        };

        // Get the tracking files for the id.
        let tracking_dir = tracking_dir(path, true).await?;
        let (tracked_paths, tracked_json) = tracking_files(&tracking_dir, &id).await?;

        // Lock tracked paths file for exclusive access
        let mut tracked_paths_file = tracked_paths_lock(&tracked_paths).await?;

        // Write JSON
        self.export(&tracked_json, None).await?;

        // Add path of document
        tracked_paths_add(&mut tracked_paths_file, path).await?;

        // Unlock the tracked paths file
        tracked_paths_unlock(tracked_paths_file).await
    }

    /// Untrack a document
    #[tracing::instrument(skip(self))]
    pub async fn untrack(&self) -> Result<()> {
        tracing::trace!("Un-tracking document");

        // Get the existing document id, and remove it, but only if it
        // starts with 'doc_'
        let id = self
            .mutate(|root| {
                const DOC: &str = "doc_";
                fn starts_with_doc(id: &Option<String>) -> bool {
                    id.as_ref()
                        .map(|id| id.starts_with(DOC))
                        .unwrap_or_default()
                }

                if let Node::Article(article) = root {
                    if starts_with_doc(&article.id) {
                        article.id.take()
                    } else {
                        article.id.clone()
                    }
                } else if let Node::Chat(chat) = root {
                    if starts_with_doc(&chat.id) {
                        chat.id.take()
                    } else {
                        chat.id.clone()
                    }
                } else if let Node::Prompt(prompt) = root {
                    if starts_with_doc(&prompt.id) {
                        prompt.id.take()
                    } else {
                        prompt.id.clone()
                    }
                } else {
                    None
                }
            })
            .await;

        // Early return if no path or id
        let Some(id) = id else {
            return Ok(());
        };
        let Some(path) = &self.path else {
            return Ok(());
        };

        // Get the closest tracking dir and return early if none found
        let tracking_dir = tracking_dir(path, false).await?;
        if !tracking_dir.exists() {
            return Ok(());
        }

        // Get the tracking files for the id.
        let (tracked_paths, tracked_json) = tracking_files(&tracking_dir, &id).await?;

        // Lock tracked paths file for exclusive access
        let mut tracked_paths_file = tracked_paths_lock(&tracked_paths).await?;

        // Remove path of document
        let has_paths = tracked_paths_remove(&mut tracked_paths_file, path).await?;

        // Remove both tracking files if no more paths in the tracked paths
        if !has_paths {
            if tracked_json.exists() {
                remove_file(&tracked_json).await?
            };
            if tracked_paths.exists() {
                remove_file(tracked_paths).await?
            }
        }

        // Unlock the tracked paths file
        tracked_paths_unlock(tracked_paths_file).await
    }

    /// Untrack the path
    pub async fn untrack_path(path: &Path) -> Result<()> {
        if path.exists() && path.is_file() && codecs::from_path_is_supported(path) {
            // Untrack the document
            let doc = Document::open(path).await?;
            doc.untrack().await?;
            doc.save().await?;

            return Ok(());
        }

        // Get the closest tracking dir and return early if none found
        let tracking_dir = tracking_dir(path, false).await?;
        if !tracking_dir.exists() {
            return Ok(());
        }

        // Given that we can't open the path to get the id (it doesn't exist of is not
        // a file) we need to iterate over all the tracked paths files and remove the
        // path from each. We could stop at the first file it is found in, but not
        // doing so is "safer".
        let mut dir_entries = read_dir(&tracking_dir).await?;
        while let Ok(Some(entry)) = dir_entries.next_entry().await {
            // For each `.paths` file
            let tracking_paths = entry.path();
            if tracking_paths.extension().unwrap_or_default() == "paths" {
                // Remove the path form the tracked paths
                let mut tracked_paths_file = tracked_paths_lock(&tracking_paths).await?;
                let has_paths = tracked_paths_remove(&mut tracked_paths_file, path).await?;

                // Remove both tracking files if no more entries in the tracking paths
                if !has_paths {
                    let id = tracking_paths
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy();
                    let json = tracking_dir.join(format!("{id}.json"));
                    if json.exists() {
                        remove_file(&json).await?
                    };

                    if tracking_paths.exists() {
                        remove_file(tracking_paths).await?
                    }
                }

                // Unlock the tracked paths file
                tracked_paths_unlock(tracked_paths_file).await?;
            }
        }

        Ok(())
    }

    /// Get the tracking status of a document
    pub async fn status(&self) -> Result<DocumentStatus> {
        // Get the path of the source file, returning early if not exists
        let Some(source) = &self.path else {
            return Ok(DocumentStatus::unsaved());
        };
        if !source.exists() {
            return Ok(DocumentStatus::unsaved());
        }

        let modified_at = modification_time(&source)?;

        // Get the document id, returning early if none
        let Some(id) = self.id().await else {
            return Ok(DocumentStatus::untracked(source, modified_at));
        };

        // Get the path to the tracked JSON, returning early if not exists
        let tracking_dir = tracking_dir(source, false).await?;
        if !tracking_dir.exists() {
            return Ok(DocumentStatus::untracked(source, modified_at));
        }
        let (.., tracked_json) = tracking_files(&tracking_dir, &id).await?;
        if !tracked_json.exists() {
            return Ok(DocumentStatus::untracked(source, modified_at));
        }

        // Get the modification time of both files
        let tracked_at = modification_time(&tracked_json)?;

        Ok(DocumentStatus::new(&source, modified_at, tracked_at, id))
    }

    /// Get the tracking status of a path
    pub async fn status_path(path: &Path) -> Result<DocumentStatus> {
        if !path.exists() {
            return Ok(DocumentStatus::deleted(path));
        }

        if !path.is_file() || !codecs::from_path_is_supported(path) {
            return Ok(DocumentStatus::unsupported(path));
        }

        let doc = Self::open(path).await?;
        doc.status().await
    }

    /// Get the tracking status of all known tracked files
    pub async fn status_tracked(path: &Path) -> Result<Vec<DocumentStatus>> {
        let tracking_dir = tracking_dir(path, false).await?;

        // Return early if no tracking file found
        if !tracking_dir.exists() {
            return Ok(Vec::new());
        }

        // Get a list of all unique tracked paths
        let mut tracked_paths = Vec::new();
        let mut dir_entries = read_dir(&tracking_dir).await?;
        while let Ok(Some(entry)) = dir_entries.next_entry().await {
            let path = entry.path();
            if path.extension().unwrap_or_default() == "paths" {
                for line in read_to_string(path).await?.lines() {
                    let path = PathBuf::from(line);
                    if !tracked_paths.contains(&path) {
                        tracked_paths.push(path)
                    }
                }
            }
        }

        // Get the the status of each
        let futures = tracked_paths.iter().map(|path| Self::status_path(&path));
        let statuses = try_join_all(futures).await?;

        Ok(statuses)
    }
}

/// Generate a new random document id
fn id_random() -> String {
    NodeId::random([b'd', b'o', b'c']).to_string()
}

/// Get the path of the closest `.stencila` directory
///
/// If the `path` is a file then starts with the parent directory of that file.
/// Walks up the directory tree until a `.stencila` directory is found.
/// If none is found, and `ensure` is true, then creates one in the starting directory.
async fn stencila_dir(path: &Path, ensure: bool) -> Result<PathBuf> {
    const STENCILA_DIR: &str = ".stencila";

    let starting_dir = if path.is_file() {
        path.parent().ok_or_eyre("File has no parent directory")?
    } else {
        path
    }
    .to_path_buf();

    // Walk up dir tree
    let mut current_dir = starting_dir.clone();
    loop {
        let stencila_dir = current_dir.join(STENCILA_DIR);
        if stencila_dir.exists() {
            return Ok(stencila_dir);
        }

        let Some(parent_dir) = current_dir.parent() else {
            break;
        };
        current_dir = parent_dir.to_path_buf();
    }

    // Not found so create one in starting dir
    let stencila_dir = starting_dir.join(STENCILA_DIR);
    if ensure {
        create_dir_all(&stencila_dir).await?;
    }

    Ok(stencila_dir)
}

/// Get the path of the closest tracking directory
async fn tracking_dir(path: &Path, ensure: bool) -> Result<PathBuf> {
    const TRACKING_DIR: &str = "tracked";

    let tracking_dir = stencila_dir(path, false).await?.join(TRACKING_DIR);

    if ensure && !tracking_dir.exists() {
        create_dir_all(&tracking_dir).await?;
    }

    Ok(tracking_dir)
}

/// Get the path of the tracking files for the document id
async fn tracking_files(tracking_dir: &Path, id: &str) -> Result<(PathBuf, PathBuf)> {
    // TODO: handle characters that are invalid - hash id?

    let tracked_paths = tracking_dir.join(format!("{id}.paths"));
    let tracked_json = tracking_dir.join(format!("{id}.json"));

    Ok((tracked_paths, tracked_json))
}

/// Open and lock a tracked paths file
async fn tracked_paths_lock(tracking_paths: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(tracking_paths)
        .await?;

    file.lock_exclusive()?;

    Ok(file)
}

/// Unlock a tracked paths file
async fn tracked_paths_unlock(file: File) -> Result<()> {
    Ok(file.unlock_async().await?)
}

/// Add a path to the tracked paths if it does not yet exist there
async fn tracked_paths_add(file: &mut File, path: &Path) -> Result<()> {
    let path = path.to_string_lossy();

    // Read the file and if none of the lines equal the path, append it
    let mut content = String::new();
    file.read_to_string(&mut content).await?;
    let has_path = content.lines().any(|line| line == path);

    // Append line if missing
    if !has_path {
        file.seek(SeekFrom::End(0)).await?;
        file.write_all([&path, "\n"].concat().as_bytes()).await?;
    }

    Ok(())
}

/// Remove a path from the tracked paths
async fn tracked_paths_remove(file: &mut File, path: &Path) -> Result<bool> {
    let path = path.to_string_lossy();

    // Read the file and filter out the lines that equal the path
    let mut old = String::new();
    file.read_to_string(&mut old).await?;
    let new = old.lines().filter(|&line| line != path).join("\n");

    let has_paths = !new.is_empty();
    if has_paths {
        // Truncate and write file
        file.set_len(0).await?;
        file.seek(SeekFrom::Start(0)).await?;
        file.write_all([&new, "\n"].concat().as_bytes()).await?;
    }

    Ok(has_paths)
}

/// Get the modification time of a path
fn modification_time(path: &Path) -> Result<u64> {
    let metadata = std::fs::File::open(path)?.metadata()?;
    Ok(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs())
}
