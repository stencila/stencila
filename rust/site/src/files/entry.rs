//! File index entry data structure

use serde::{Deserialize, Serialize};

/// A single file entry in the files index
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    /// Relative path from site root (e.g., "docs/guide/data.csv")
    pub path: String,

    /// File size in bytes
    pub size: u64,

    /// File extension (without leading dot, lowercase)
    pub extension: String,

    /// Last modified timestamp (ISO 8601)
    pub last_modified: String,
}

impl FileEntry {
    /// Create a new file entry
    pub fn new(path: String, size: u64, extension: String, last_modified: String) -> Self {
        Self {
            path,
            size,
            extension,
            last_modified,
        }
    }
}
