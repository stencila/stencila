use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use fs4::tokio::AsyncFileExt;

use common::{
    eyre::{bail, Result},
    itertools::Itertools,
    serde::Serialize,
    serde_with::skip_serializing_none,
    strum::Display,
    tokio::{
        fs::{create_dir_all, remove_file, File, OpenOptions},
        io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom},
    },
    tracing,
};
use schema::{Node, NodeId};

use crate::Document;

#[derive(Default, Display, Serialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub enum DocumentStatusFlag {
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

        // Get the tracking files for the id. The paths file is locked for
        // exclusive access
        let (paths, json) = tracking_files(&path, &id).await?;
        let mut paths_file = tracking_paths_lock(&paths).await?;

        // Write JSON
        self.export(&json, None).await?;

        // Add path of document
        tracking_paths_add(&mut paths_file, path).await?;

        // Unlock the paths file
        tracking_paths_unlock(paths_file).await
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

        // Get the tracking files for the id. The paths file is locked for
        // exclusive access
        let (paths, json) = tracking_files(&path, &id).await?;
        let mut paths_file = tracking_paths_lock(&paths).await?;

        // Remove path of document
        let has_paths = tracking_paths_remove(&mut paths_file, path).await?;

        // Remove the tracking files if no more entries in the tracking list
        if !has_paths {
            if json.exists() {
                remove_file(&json).await?
            };
            if paths.exists() {
                remove_file(paths).await?
            }
        }

        // Unlock the paths file
        tracking_paths_unlock(paths_file).await
    }

    /// Get the status of a tracked document
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

        // Get the path to the tracking file, returning early if not exists
        let (.., json) = tracking_files(source, &id).await?;
        if !json.exists() {
            return Ok(DocumentStatus::untracked(source, modified_at));
        }

        // Get the modification time of both files
        let tracked_at = modification_time(&json)?;

        Ok(DocumentStatus::new(&source, modified_at, tracked_at, id))
    }

    pub async fn status_of(path: &Path) -> Result<DocumentStatus> {
        if !path.is_file() || !codecs::from_path_is_supported(path) {
            return Ok(DocumentStatus::unsupported(path));
        }

        let doc = Self::open(path).await?;
        doc.status().await
    }
}

/// Generate a new random document id
fn id_random() -> String {
    NodeId::random([b'd', b'o', b'c']).to_string()
}

/// Get the directory of tracked documents
async fn tracking_dir(path: &Path) -> Result<PathBuf> {
    // TODO: walk up directory tree from path, until first '.stencila' directory
    // is found. If none found then use current dir.
    let dir = PathBuf::from(".stencila/tracked");

    if !dir.exists() {
        create_dir_all(&dir).await?;
    }

    Ok(dir)
}

/// Get the path of the tracking files for the document id
async fn tracking_files(path: &Path, id: &str) -> Result<(PathBuf, PathBuf)> {
    let dir = tracking_dir(path).await?;

    // TODO: handle characters that are invalid - hash id?

    let paths = dir.join(format!("{id}.paths"));
    let json = dir.join(format!("{id}.json"));

    Ok((paths, json))
}

/// Lock a paths tracking file
async fn tracking_paths_lock(path: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .await?;

    file.lock_exclusive()?;

    Ok(file)
}

/// Unlock a paths tracking file
async fn tracking_paths_unlock(file: File) -> Result<()> {
    Ok(file.unlock_async().await?)
}

/// Add a path to the paths tracking file if it does not yet exist there
async fn tracking_paths_add(file: &mut File, path: &Path) -> Result<()> {
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

/// Remove a path from the paths tracking file
async fn tracking_paths_remove(file: &mut File, path: &Path) -> Result<bool> {
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
