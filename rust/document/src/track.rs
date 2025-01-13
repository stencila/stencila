use std::{
    cmp::Ordering,
    env::current_dir,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use fs4::tokio::AsyncFileExt;

use common::{
    eyre::{bail, OptionExt, Result},
    futures::future::try_join_all,
    itertools::Itertools,
    once_cell::sync::Lazy,
    regex::Regex,
    seahash::SeaHasher,
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
    /// The file is of a format that does not support tracking
    #[default]
    Unsupported,

    /// The file is not tracked: it does not have a document id,
    /// or if it does there is no tracking file for it
    Untracked,

    /// The document is currently in-memory only so it can not
    /// yet be tracked
    Unsaved,

    /// There is an entry for the file in a tracked paths file
    /// but that path no longer exists in the tracked directory
    Deleted,

    /// There is an entry for the file in a tracked paths file,
    /// and the file exists, but it is has no document Id(String),
    IdMissing(String),

    /// There is an entry for the file in a tracked paths file,
    /// and the file exists, but it is has a different document id.
    IdDifferent(String, String),

    /// The file is ahead of the tracking file: it has changed
    /// to it since it was last synced
    Ahead,

    /// The file is behind the tracking file: there have been
    /// changes to a linked file which have not been propagated
    /// to the file
    Behind,

    /// The file is synced with the tracking file: they have the
    /// same modification time
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

    fn id_missing(path: &Path, doc_id: &str) -> Self {
        Self {
            status: DocumentStatusFlag::IdMissing(doc_id.into()),
            path: Some(path.to_path_buf()),
            ..Default::default()
        }
    }

    fn id_different(path: &Path, doc_id: &str, found_doc_id: &str) -> Self {
        Self {
            status: DocumentStatusFlag::IdDifferent(doc_id.into(), found_doc_id.into()),
            path: Some(path.to_path_buf()),
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

    fn untracked(path: &Path, modified_at: u64, doc_id: Option<String>) -> Self {
        Self {
            status: DocumentStatusFlag::Untracked,
            path: Some(path.to_path_buf()),
            modified_at: Some(modified_at),
            doc_id,
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

        // Get the tracking files for the id
        let tracking_dir = tracking_dir(path, true).await?;
        let (tracked_paths, tracked_json) = tracking_files(&tracking_dir, &id).await?;

        // Lock tracked paths file for exclusive access
        let mut tracked_paths_file = tracked_paths_lock(&tracked_paths).await?;

        // Write JSON
        self.export(&tracked_json, None).await?;

        // Add path of document
        tracked_paths_add(&tracking_dir, &mut tracked_paths_file, path).await?;

        // Unlock the tracked paths file
        tracked_paths_unlock(tracked_paths_file).await
    }

    /// Track a document
    #[tracing::instrument]
    pub async fn track_path(path: &Path) -> Result<()> {
        let doc = Document::open(path).await?;
        doc.track().await?;
        doc.save().await?;

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
        let has_paths = tracked_paths_remove(&tracking_dir, &mut tracked_paths_file, path).await?;

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
    ///
    /// This is called from outside the document and so allows for more
    /// removing tracking files for document files that have been deleted or
    /// have broken ids.
    pub async fn untrack_path(path: &Path) -> Result<()> {
        if path.exists() && path.is_file() && codecs::from_path_is_supported(path) {
            // Untrack the document
            let doc = Document::open(path).await?;
            doc.untrack().await?;
            doc.save().await?;

            // It is tempting to return early here, but for documents with
            // missing or different ids from those in the tracking directory
            // we need to continue so that can be cleaned up.
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
                let has_paths =
                    tracked_paths_remove(&tracking_dir, &mut tracked_paths_file, path).await?;

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
        let Some(path) = &self.path else {
            return Ok(DocumentStatus::unsaved());
        };
        if !path.exists() {
            return Ok(DocumentStatus::unsaved());
        }

        // Get the modification time of file
        let modified_at = modification_time(&path)?;

        // Get the document id, returning early if none
        let Some(id) = self.id().await else {
            return Ok(DocumentStatus::untracked(path, modified_at, None));
        };

        // Get the path to the tracked JSON, returning early if it does not exist
        let tracking_dir = tracking_dir(path, false).await?;
        if !tracking_dir.exists() {
            return Ok(DocumentStatus::untracked(path, modified_at, Some(id)));
        }
        let (.., tracked_json) = tracking_files(&tracking_dir, &id).await?;
        if !tracked_json.exists() {
            return Ok(DocumentStatus::untracked(path, modified_at, Some(id)));
        }

        // Get the modification time of tracked JSON
        let tracked_at = modification_time(&tracked_json)?;

        Ok(DocumentStatus::new(&path, modified_at, tracked_at, id))
    }

    /// Get the tracking status of a path
    ///
    /// This is called from outside the document and so allows for more
    /// granularity of status flags such a `Deleted`.
    ///
    /// If an expected document id is provided it can also return
    /// `IdMissing` or `IdDifferent`.
    pub async fn status_path(
        path: PathBuf,
        expected_doc_id: Option<String>,
    ) -> Result<DocumentStatus> {
        if !path.exists() {
            return Ok(DocumentStatus::deleted(&path));
        }

        if !path.is_file() || !codecs::from_path_is_supported(&path) {
            return Ok(DocumentStatus::unsupported(&path));
        }

        let doc = Self::open(&path).await?;
        let status = doc.status().await?;

        if let Some(expected_doc_id) = &expected_doc_id {
            if let Some(found_doc_id) = &status.doc_id {
                if found_doc_id != expected_doc_id {
                    return Ok(DocumentStatus::id_different(
                        &path,
                        expected_doc_id,
                        found_doc_id,
                    ));
                }
            } else {
                return Ok(DocumentStatus::id_missing(&path, expected_doc_id));
            }
        }

        Ok(status)
    }

    /// Get the tracking status of all known tracked files
    pub async fn status_tracked(path: &Path) -> Result<Vec<DocumentStatus>> {
        let tracking_dir = tracking_dir(path, false).await?;

        // Return early if no tracking dir found
        if !tracking_dir.exists() {
            tracing::debug!("No tracking dir found for {path:?}");
            return Ok(Vec::new());
        }

        tracing::debug!("Tracking dir for {path:?}: {tracking_dir:?}");

        // Paths need to be made relative to the parent of the `.stencila` directory
        let tracked_dir = tracked_dir(&tracking_dir)?;

        // Get a list of all tracked path / doc id combinations
        let mut tracked_paths = Vec::new();
        let mut dir_entries = read_dir(&tracking_dir).await?;
        while let Ok(Some(entry)) = dir_entries.next_entry().await {
            let path = entry.path();
            if path.extension().unwrap_or_default() == "paths" {
                let Some(doc_id) = path.file_stem() else {
                    continue;
                };
                let doc_id = doc_id.to_string_lossy().to_string();
                for line in read_to_string(path).await?.lines() {
                    let path = tracked_dir.join(line);
                    tracked_paths.push((path, doc_id.clone()))
                }
            }
        }
        tracked_paths.sort();

        // Get the the status of each tracked path
        let futures = tracked_paths
            .into_iter()
            .map(|(path, doc_id)| async move { Self::status_path(path, Some(doc_id)).await });
        let mut statuses = try_join_all(futures).await?;

        // Make the paths in the statuses relative to the tracked dir
        for status in statuses.iter_mut() {
            if let Some(path) = status.path.clone() {
                status.path = path
                    .strip_prefix(tracked_dir)
                    .map(|path| path.to_path_buf())
                    .ok();
            }
        }

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
/// Walks up the directory tree until a `.stencila` or `.git` directory is found.
/// If none is found, and `ensure` is true, then creates one, next to the `.git`
/// directory if any, or in the starting directory.
async fn stencila_dir(path: &Path, ensure: bool) -> Result<PathBuf> {
    const STENCILA_DIR: &str = ".stencila";

    // Get a canonicalized starting path
    // This allows for accepting files that do not exist by finding the
    // closest ancestor dir that does exist. This is necessary when a
    // user wants to untrack a deleted file, possibly in a subdir of the current dir
    let mut starting_path = path.to_path_buf();
    loop {
        match starting_path.canonicalize() {
            Ok(path) => {
                starting_path = path;
                break;
            }
            Err(..) => {
                starting_path = match starting_path.parent() {
                    Some(path) => path.to_path_buf(),
                    None => current_dir()?,
                }
            }
        }
    }

    let starting_dir = if starting_path.is_file() {
        starting_path
            .parent()
            .ok_or_eyre("File has no parent directory")?
            .to_path_buf()
    } else {
        starting_path
    };

    // Walk up dir tree
    let mut current_dir = starting_dir.clone();
    loop {
        let stencila_dir = current_dir.join(STENCILA_DIR);
        if stencila_dir.exists() {
            return Ok(stencila_dir);
        }

        if ensure {
            let git_dir = current_dir.join(".git");
            if git_dir.exists() {
                create_dir_all(&stencila_dir).await?;
                return Ok(stencila_dir);
            }
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

    // Note: must call `stencila_dir` with ensure even through checking for ensure
    // below, to ensure that walk stops at closes Git repo
    let tracking_dir = stencila_dir(path, true).await?.join(TRACKING_DIR);

    if ensure && !tracking_dir.exists() {
        create_dir_all(&tracking_dir).await?;
    }

    Ok(tracking_dir)
}

/// Get the path of the tracked directory from the path of the tracking directory
fn tracked_dir(path: &Path) -> Result<&Path> {
    path.parent()
        .and_then(|path| path.parent())
        .ok_or_eyre("Unable to get tracked directory")
}

/// Get the path of the tracking files for the document id
async fn tracking_files(tracking_dir: &Path, id: &str) -> Result<(PathBuf, PathBuf)> {
    let file_stem = file_stem_for_id(id);

    let tracked_paths = tracking_dir.join(format!("{file_stem}.paths"));
    let tracked_json = tracking_dir.join(format!("{file_stem}.json"));

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
async fn tracked_paths_add(tracking_dir: &Path, file: &mut File, path: &Path) -> Result<()> {
    // Get the path relative to the tracked directory
    let tracked_dir = tracked_dir(tracking_dir)?;
    let relative_path = path
        .canonicalize()?
        .strip_prefix(tracked_dir)?
        .to_path_buf();
    let relative_path = relative_path.to_string_lossy();

    // Read the file and if none of the lines equal the path, append it
    let mut content = String::new();
    file.read_to_string(&mut content).await?;
    let has_path = content.lines().any(|line| line == relative_path);

    // Append line if missing
    if !has_path {
        file.seek(SeekFrom::End(0)).await?;
        file.write_all([&relative_path, "\n"].concat().as_bytes())
            .await?;
    }

    Ok(())
}

/// Remove a path from the tracked paths
async fn tracked_paths_remove(tracking_dir: &Path, file: &mut File, path: &Path) -> Result<bool> {
    // Get the path relative to the tracked directory
    // Note: this allows for paths that no longer exist i.e. removing a tracked file
    // that has been deleted
    let tracked_dir = tracked_dir(tracking_dir)?;
    let relative_path = match path.canonicalize() {
        Ok(path) => path.strip_prefix(tracked_dir)?.to_path_buf(),
        Err(..) => path.to_path_buf(),
    };
    let relative_path = relative_path.to_string_lossy();

    // Read the file and filter out the lines that equal the path
    let mut old_paths = String::new();
    file.read_to_string(&mut old_paths).await?;
    let new_paths = old_paths
        .lines()
        .filter(|&line| line != relative_path)
        .join("\n");

    let has_paths = !new_paths.is_empty();
    if has_paths {
        // Truncate and write file
        file.set_len(0).await?;
        file.seek(SeekFrom::Start(0)).await?;
        file.write_all([&new_paths, "\n"].concat().as_bytes())
            .await?;
    }

    Ok(has_paths)
}

/// Get the modification time of a path
fn modification_time(path: &Path) -> Result<u64> {
    let metadata = std::fs::File::open(path)?.metadata()?;
    Ok(metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs())
}

/// Generate a valid filename for an id
///
/// If an invalid characters replaces them with hyphen and
/// add a hash for uniqueness.
pub fn file_stem_for_id(id: &str) -> String {
    static INVALID_CHARS: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"[\s.<>:"/\\|?*\x00-\x1F]"#).expect("invalid regex"));

    // Sanitize by replacing invalid chars with hyphens
    let sanitized = INVALID_CHARS.replace_all(id, "-");

    // If no sanitation, just return original
    if sanitized == id {
        return id.to_string();
    }

    // Create hash for uniqueness
    let mut hasher = SeaHasher::new();
    id.as_bytes().hash(&mut hasher);
    let hash = format!("{:.8x}", hasher.finish());

    // Combine parts with length limit
    let filename = if sanitized.len() > 32 {
        format!("{}-{}", &sanitized[..32], hash)
    } else {
        format!("{}-{}", sanitized, hash)
    };

    filename
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename_sanitization() {
        assert_eq!(file_stem_for_id("doc_123"), "doc_123");

        assert_eq!(file_stem_for_id(""), "");
        assert_eq!(file_stem_for_id(" "), "--7d017dd9b4dd6a17");
        assert_eq!(file_stem_for_id("  "), "---bc60483fa376d879");

        assert_eq!(
            file_stem_for_id("hello/world"),
            "hello-world-98441892c2bff380"
        );
        assert_eq!(
            file_stem_for_id("file*name?foo.bar"),
            "file-name-foo-bar-f2d200db1d4eff93"
        );
        assert_eq!(
            file_stem_for_id("file/name\\foo\"bar"),
            "file-name-foo-bar-1e8631e402c9df43"
        );
    }
}
