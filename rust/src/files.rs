use schemars::JsonSchema;
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

/// # A file or directory within a `Project`
#[skip_serializing_none]
#[derive(Debug, Default, Clone, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    /// The relative path of the file within the project folder
    pub path: String,

    /// Time that the file was last modified (seconds since Unix Epoch)
    pub modified: Option<u64>,

    /// Size of the file in bytes
    pub size: Option<u64>,

    /// The media type (aka MIME type) of the file
    pub media_type: Option<String>,

    /// The SHA1 hash of the contents of the file
    pub sha1: Option<String>,

    /// The parent `File`, if any
    pub parent: Option<PathBuf>,

    /// If a directory, a list of the canonical paths of the files within it.
    /// Otherwise, `None`.
    pub children: Option<Vec<PathBuf>>,
}

impl File {
    pub fn from_path(folder: &Path, path: &Path) -> (PathBuf, File) {
        let canonical_path = path.canonicalize().expect("Unable to canonicalize path");
        let relative_path = path
            .strip_prefix(folder)
            .expect("Unable to strip prefix")
            .display()
            .to_string();

        let (modified, size) = match path.metadata() {
            Ok(metadata) => {
                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .and_then(|duration| Some(duration.as_secs()));
                let size = Some(metadata.len());
                (modified, size)
            }
            Err(_) => (None, None),
        };

        let (media_type, children) = if path.is_file() {
            let media_type = mime_guess::from_path(path)
                .first()
                .map(|mime| mime.essence_str().to_string());

            (media_type, None)
        } else {
            (None, Some(Vec::new()))
        };

        let file = File {
            path: relative_path,
            modified,
            size,
            media_type,
            children,
            ..Default::default()
        };

        (canonical_path, file)
    }
}

/// # The set of `File`s within a `Project`
#[derive(Debug, Default, Clone, JsonSchema, Serialize)]
#[serde(transparent)]
pub struct Files {
    pub files: BTreeMap<PathBuf, File>,
}

impl Files {
    pub fn from_path(path: &Path) -> Files {
        // Collect all the files
        let mut files = walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|entry| {
                let entry = match entry.ok() {
                    Some(entry) => entry,
                    None => return None,
                };
                Some(File::from_path(path, entry.path()))
            })
            .into_iter()
            .collect::<BTreeMap<PathBuf, File>>();

        // Resolve `parent` and `children` properties
        // This needs to clone the files to avoid mutable borrow twice,
        // there may be a more efficient alternative
        let mut children: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
        for (path, file) in &mut files {
            if let Some(parent_path) = path.parent() {
                let parent_path = parent_path.to_path_buf();
                file.parent = Some(parent_path.clone());
                children
                    .entry(parent_path)
                    .or_insert_with(Vec::new)
                    .push(path.clone());
            }
        }
        for (path, vec) in children {
            if let Some(parent) = files.get_mut(&path) {
                parent.children = Some(vec)
            }
        }

        Files { files }
    }
}
