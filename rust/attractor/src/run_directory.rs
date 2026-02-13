//! Run directory abstraction (§5.6).
//!
//! Each pipeline execution gets its own run directory containing a
//! manifest, checkpoint, and per-node status files. This module provides
//! creation and I/O helpers for that directory structure.
//!
//! # Deviation from spec
//!
//! The spec (§5.6) describes per-node artifacts at `{run_root}/{node_id}/...`.
//! This implementation uses `{run_root}/nodes/{node_id}/...` instead, placing
//! all node directories under a `nodes/` subdirectory. This avoids mixing
//! per-node directories with root-level files (`manifest.json`,
//! `checkpoint.json`) and makes directory listing cleaner.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AttractorResult;
use crate::types::Outcome;

/// Metadata about a pipeline run, serialized as `manifest.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    /// Name of the pipeline graph.
    pub name: String,
    /// Goal description from graph attributes.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub goal: String,
    /// ISO 8601 timestamp when the run started.
    pub start_time: String,
}

/// A run directory for a single pipeline execution.
///
/// Directory structure:
/// ```text
/// <root>/
///   manifest.json
///   checkpoint.json
///   nodes/
///     <node_id>/
///       status.json
/// ```
#[derive(Debug, Clone)]
pub struct RunDirectory {
    root: PathBuf,
}

impl RunDirectory {
    /// Create a new run directory at the given root path.
    ///
    /// Creates the directory and the `nodes/` subdirectory.
    ///
    /// # Errors
    ///
    /// Returns an error if the directories cannot be created.
    pub fn create(root: impl Into<PathBuf>) -> AttractorResult<Self> {
        let root = root.into();
        std::fs::create_dir_all(root.join("nodes"))?;
        Ok(Self { root })
    }

    /// Wrap an existing run directory path without creating directories.
    ///
    /// Use this when the directory was already created (e.g., by the engine)
    /// and you only need path helpers and I/O methods.
    #[must_use]
    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Path to the manifest file.
    #[must_use]
    pub fn manifest_path(&self) -> PathBuf {
        self.root.join("manifest.json")
    }

    /// Path to the checkpoint file.
    #[must_use]
    pub fn checkpoint_path(&self) -> PathBuf {
        self.root.join("checkpoint.json")
    }

    /// Path to a node's directory.
    #[must_use]
    pub fn node_dir(&self, node_id: &str) -> PathBuf {
        self.root.join("nodes").join(node_id)
    }

    /// Path to a node's status file.
    #[must_use]
    pub fn status_path(&self, node_id: &str) -> PathBuf {
        self.node_dir(node_id).join("status.json")
    }

    /// The root path of this run directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Write the manifest to `manifest.json`.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn write_manifest(&self, manifest: &Manifest) -> AttractorResult<()> {
        let json = serde_json::to_string_pretty(manifest)?;
        std::fs::write(self.manifest_path(), json)?;
        Ok(())
    }

    /// Write a node's outcome to its `status.json`.
    ///
    /// Creates the node directory if it does not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn write_status(&self, node_id: &str, outcome: &Outcome) -> AttractorResult<()> {
        let dir = self.node_dir(node_id);
        std::fs::create_dir_all(&dir)?;
        let json = serde_json::to_string_pretty(outcome)?;
        std::fs::write(self.status_path(node_id), json)?;
        Ok(())
    }

    /// Read a node's outcome from its `status.json`.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or contains invalid JSON.
    pub fn read_status(&self, node_id: &str) -> AttractorResult<Outcome> {
        let data = std::fs::read_to_string(self.status_path(node_id))?;
        let outcome: Outcome = serde_json::from_str(&data)?;
        Ok(outcome)
    }
}
