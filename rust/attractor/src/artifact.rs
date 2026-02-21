//! Artifact store (§5.5).
//!
//! Named, typed storage for large stage outputs that do not belong
//! in the context. Artifacts below 100KB are stored in memory; above
//! that threshold they are written to disk.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

use crate::error::{AttractorError, AttractorResult};

/// Default file-backing threshold in bytes (100KB).
const FILE_BACKING_THRESHOLD: usize = 100 * 1024;

/// Metadata about a stored artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactInfo {
    /// Unique identifier for this artifact.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Size in bytes.
    pub size_bytes: usize,
    /// Whether this artifact is stored on disk.
    pub is_file_backed: bool,
}

/// Storage location for an artifact.
#[derive(Debug, Clone)]
enum ArtifactData {
    /// Stored in memory.
    InMemory(Vec<u8>),
    /// Stored on disk at this path.
    FileBacked(PathBuf),
}

/// A thread-safe store for pipeline artifacts.
#[derive(Debug)]
pub struct ArtifactStore {
    artifacts: RwLock<HashMap<String, (ArtifactInfo, ArtifactData)>>,
    base_dir: Option<PathBuf>,
    #[cfg(feature = "sqlite")]
    sqlite_backend: Option<crate::sqlite_backend::SqliteBackend>,
    #[cfg(feature = "sqlite")]
    workspace_root: Option<PathBuf>,
}

impl ArtifactStore {
    /// Create a new artifact store.
    ///
    /// If `base_dir` is provided, large artifacts will be written to
    /// `{base_dir}/artifacts/`. Otherwise, all artifacts are in-memory.
    #[must_use]
    pub fn new(base_dir: Option<PathBuf>) -> Self {
        Self {
            artifacts: RwLock::new(HashMap::new()),
            base_dir,
            #[cfg(feature = "sqlite")]
            sqlite_backend: None,
            #[cfg(feature = "sqlite")]
            workspace_root: None,
        }
    }

    /// Create a store that also registers persisted artifacts in SQLite.
    #[cfg(feature = "sqlite")]
    #[must_use]
    pub fn with_sqlite(
        base_dir: Option<PathBuf>,
        sqlite_backend: crate::sqlite_backend::SqliteBackend,
        workspace_root: PathBuf,
    ) -> Self {
        Self {
            artifacts: RwLock::new(HashMap::new()),
            base_dir,
            sqlite_backend: Some(sqlite_backend),
            workspace_root: Some(workspace_root),
        }
    }

    /// Store an artifact by ID and name.
    ///
    /// # Errors
    ///
    /// Returns an error if file-backed storage fails.
    pub fn store(
        &self,
        artifact_id: impl Into<String>,
        name: impl Into<String>,
        data: &[u8],
    ) -> AttractorResult<ArtifactInfo> {
        let id = artifact_id.into();
        let name = name.into();
        let size = data.len();
        #[cfg(feature = "sqlite")]
        let force_file_backed = self.sqlite_backend.is_some();
        #[cfg(not(feature = "sqlite"))]
        let force_file_backed = false;

        let is_file_backed =
            (size > FILE_BACKING_THRESHOLD || force_file_backed) && self.base_dir.is_some();

        let stored_data = if let (true, Some(base)) = (is_file_backed, &self.base_dir) {
            let dir = base.join("artifacts");
            std::fs::create_dir_all(&dir)?;
            let path = dir.join(format!("{id}.json"));
            std::fs::write(&path, data)?;
            ArtifactData::FileBacked(path)
        } else {
            ArtifactData::InMemory(data.to_vec())
        };

        let info = ArtifactInfo {
            id: id.clone(),
            name,
            size_bytes: size,
            is_file_backed,
        };

        let mut artifacts = self
            .artifacts
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Clean up old file-backed data before overwriting the entry
        if let Some((_, ArtifactData::FileBacked(old_path))) = artifacts.get(&id) {
            let _ = std::fs::remove_file(old_path);
        }
        artifacts.insert(id, (info.clone(), stored_data));
        #[cfg(feature = "sqlite")]
        self.register_sqlite_artifact(&info, artifacts.get(&info.id).map(|(_, data)| data));
        Ok(info)
    }

    /// Retrieve an artifact's data by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the artifact is not found or file I/O fails.
    pub fn retrieve(&self, artifact_id: &str) -> AttractorResult<Vec<u8>> {
        let artifacts = self
            .artifacts
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (_, data) =
            artifacts
                .get(artifact_id)
                .ok_or_else(|| AttractorError::InvalidPipeline {
                    reason: format!("artifact not found: {artifact_id}"),
                })?;
        match data {
            ArtifactData::InMemory(bytes) => Ok(bytes.clone()),
            ArtifactData::FileBacked(path) => Ok(std::fs::read(path)?),
        }
    }

    /// Check if an artifact exists.
    #[must_use]
    pub fn has(&self, artifact_id: &str) -> bool {
        self.artifacts
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(artifact_id)
    }

    /// List all stored artifacts.
    #[must_use]
    pub fn list(&self) -> Vec<ArtifactInfo> {
        self.artifacts
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .map(|(info, _)| info.clone())
            .collect()
    }

    /// Remove an artifact by ID.
    ///
    /// If the artifact was file-backed, the file is deleted.
    pub fn remove(&self, artifact_id: &str) {
        let mut artifacts = self
            .artifacts
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((_, ArtifactData::FileBacked(path))) = artifacts.remove(artifact_id) {
            let _ = std::fs::remove_file(path);
        }
    }

    /// Remove all artifacts.
    pub fn clear(&self) {
        let mut artifacts = self
            .artifacts
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (_, data) in artifacts.values() {
            if let ArtifactData::FileBacked(path) = data {
                let _ = std::fs::remove_file(path);
            }
        }
        artifacts.clear();
    }

    /// Return the base directory for file-backed artifacts, if configured.
    #[must_use]
    pub fn base_dir(&self) -> Option<&Path> {
        self.base_dir.as_deref()
    }

    #[cfg(feature = "sqlite")]
    fn register_sqlite_artifact(&self, info: &ArtifactInfo, data: Option<&ArtifactData>) {
        let (Some(backend), Some(workspace_root), Some(data)) =
            (&self.sqlite_backend, &self.workspace_root, data)
        else {
            return;
        };

        let ArtifactData::FileBacked(path) = data else {
            return;
        };

        let relative = path
            .canonicalize()
            .ok()
            .and_then(|p| {
                workspace_root
                    .canonicalize()
                    .ok()
                    .and_then(|root| p.strip_prefix(root).ok().map(|rel| rel.to_path_buf()))
            })
            .unwrap_or_else(|| path.clone());

        if let Err(error) = backend.insert_artifact(
            &info.id,
            &info.name,
            None,
            i64::try_from(info.size_bytes).ok(),
            &relative.to_string_lossy(),
        ) {
            tracing::warn!("Failed to register artifact in SQLite: {error}");
        }
    }
}
