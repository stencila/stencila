//! Files index generation for static sites
//!
//! This module generates a files index during site rendering that can be
//! used by the upload component to browse existing files.
//!
//! The index is organized by directory, with one JSON file per directory
//! containing file metadata. This allows efficient O(1) lookup of files
//! in a specific directory.

mod entry;
mod generate;

pub use entry::FileEntry;
pub use generate::{FilesIndexStats, generate_files_index};
