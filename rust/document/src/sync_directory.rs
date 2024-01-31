use std::path::{Component, Path, PathBuf};

use common::{
    eyre::{bail, Result},
    serde::{Deserialize, Serialize},
    tokio::{
        self,
        fs::{create_dir_all, remove_dir_all, remove_file, rename, File},
        sync::mpsc::{Receiver, Sender},
    },
    tracing,
};

use crate::Document;

/// An action to apply to a path within a directory
#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "common::serde")]
pub struct DirectoryAction {
    /// The type of action
    r#type: DirectoryActionType,

    /// The path to the file or subdirectory to which the action should be applied
    path: PathBuf,

    /// The new path for rename actions
    to: Option<PathBuf>,
}

/// The type of a `DirectoryAction`
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", crate = "common::serde")]
enum DirectoryActionType {
    CreateFile,
    CreateDirectory,
    Delete,
    Rename,
}

/// A serializable error that can be sent to the client when an action fails
#[derive(Debug, Serialize)]
#[serde(crate = "common::serde")]
pub struct DirectoryActionError {
    /// The action that caused the error
    action: DirectoryAction,

    /// The error message
    message: String,
}

impl DirectoryAction {
    /// Apply a directory action to the file system
    async fn apply(&self, dir: &Path) -> Result<()> {
        /// Check if a path attempts traversal
        fn traverses(path: &Path) -> bool {
            path.components()
                .any(|component| matches!(component, Component::ParentDir))
        }

        let path = dir.join(&self.path);
        if traverses(&path) {
            bail!("Attempting to perform an action outside of directory");
        }

        use DirectoryActionType::*;
        match self.r#type {
            CreateFile => {
                if !path.exists() {
                    File::create(path).await?;
                } else {
                    bail!("Unable to create file; path already exists");
                }
            }
            CreateDirectory => {
                if !path.exists() {
                    create_dir_all(path).await?;
                } else {
                    bail!("Unable to create directory; path already exists");
                }
            }
            Rename => {
                if !path.exists() {
                    bail!("Unable to rename/move path; path does not exist");
                }

                if let Some(to) = &self.to {
                    if traverses(to) {
                        bail!("Attempting to perform an action outside of directory");
                    }
                    let to = dir.join(to);
                    rename(&path, to).await?;
                } else {
                    bail!("Unable to rename/move path; destination path not provided")
                }
            }
            Delete => {
                if !path.exists() {
                    bail!("Unable to delete path; path does not exist");
                }

                if path.is_file() {
                    remove_file(&path).await?;
                } else if path.is_dir() {
                    remove_dir_all(&path).await?;
                }
            }
        }

        Ok(())
    }
}

impl Document {
    /// Receive directory actions and apply them to a file system directory
    #[tracing::instrument(skip_all)]
    pub async fn sync_directory(
        &self,
        dir: PathBuf,
        mut action_receiver: Receiver<DirectoryAction>,
        error_sender: Sender<DirectoryActionError>,
    ) -> Result<()> {
        tracing::trace!("Syncing directory with actions");

        let dir = dir.to_path_buf();
        tokio::spawn(async move {
            while let Some(action) = action_receiver.recv().await {
                tracing::trace!("Received directory action");

                if let Err(error) = action.apply(&dir).await {
                    tracing::error!("While applying directory action: {error}");

                    let error = DirectoryActionError {
                        action,
                        message: error.to_string(),
                    };
                    if let Err(error) = error_sender.send(error).await {
                        tracing::error!("While sending directory action error: {error}");
                    }
                }
            }
        });

        Ok(())
    }
}
