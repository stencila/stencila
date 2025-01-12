use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use common::{
    eyre::{bail, Result},
    serde::Serialize,
    serde_with::skip_serializing_none,
    strum::Display,
    tokio::fs::{create_dir_all, remove_file},
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

        // Ensure that there is a tracking file
        let file = tracking_file(&id).await?;
        if !file.exists() {
            self.export(&file, None).await?;
        }

        // TODO: add the path to the tracking list

        Ok(())
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
        let Some(id) = id else {
            return Ok(());
        };

        // TODO: remove path from the tracking list

        // Remove the tracking file if no more entries in the tracking list
        let file = tracking_file(&id).await?;
        if file.exists() {
            remove_file(&file).await?;
        }

        Ok(())
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
        let tracking = tracking_file(&id).await?;
        if !tracking.exists() {
            return Ok(DocumentStatus::untracked(source, modified_at));
        }

        // Get the modification time of both files
        let tracked_at = modification_time(&tracking)?;

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
async fn tracking_dir() -> Result<PathBuf> {
    let dir = PathBuf::from(".stencila/tracked");

    if !dir.exists() {
        create_dir_all(&dir).await?;
    }

    Ok(dir)
}

/// Get the path of the tracking file
async fn tracking_file(id: &str) -> Result<PathBuf> {
    // TODO: handle characters that are invalid - hash id?
    Ok(tracking_dir().await?.join(format!("{id}.json")))
}

/// Get the modification time of a path
fn modification_time(path: &Path) -> Result<u64> {
    let metadata = std::fs::File::open(path)?.metadata()?;
    Ok(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs())
}
