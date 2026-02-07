//! Manifest: the source of truth for which segments exist and their integrity.
//!
//! The manifest is a JSON file (`manifest.json`) stored in the `.corpus/`
//! directory. It tracks all segments, their blake3 hashes (once sealed),
//! and the schema version. It also serves as the sync primitive — comparing
//! two manifests produces a [`ManifestDiff`] that tells a sync layer exactly
//! which segments need to be uploaded or downloaded.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::schema;
use crate::segment::SegmentId;

/// The manifest file name.
const MANIFEST_FILE: &str = "manifest.json";

// ---------------------------------------------------------------------------
// Manifest
// ---------------------------------------------------------------------------

/// Top-level manifest structure, serialized as `manifest.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Monotonically increasing. Bumped on every build that modifies segments.
    pub version: u64,

    /// Schema version applied to all segments.
    pub schema_version: String,

    /// Ordered list of segments (oldest first).
    pub segments: Vec<SegmentMeta>,

    /// ID of the currently active (unsealed) segment, if any.
    pub active_segment: Option<SegmentId>,
}

/// Metadata for a single segment within the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentMeta {
    pub id: SegmentId,

    /// blake3 hash of the sealed segment file. `None` for the active segment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,

    /// Byte size of the segment file.
    pub size: u64,

    /// Number of chunks in this segment.
    pub chunk_count: u64,

    /// RFC 3339 timestamp when the segment was sealed. `None` for active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sealed_at: Option<String>,
}

impl SegmentMeta {
    /// Create metadata for a brand-new active segment (no hash, not sealed).
    pub fn active(id: SegmentId) -> Self {
        Self {
            id,
            hash: None,
            size: 0,
            chunk_count: 0,
            sealed_at: None,
        }
    }
}

// ---------------------------------------------------------------------------
// ManifestDiff — sync primitive
// ---------------------------------------------------------------------------

/// The difference between two manifests. A sync layer consumes this to know
/// which segments to upload/download.
#[derive(Debug, Clone)]
pub struct ManifestDiff {
    /// Segments present in `self` (local) but not in `other` (remote).
    /// These need to be uploaded on push.
    pub added: Vec<SegmentMeta>,

    /// Segment IDs present in `other` but not in `self`.
    /// These were removed locally (or are new on remote for pull).
    pub removed: Vec<SegmentId>,

    /// Whether the active segment changed.
    pub active_changed: bool,
}

// ---------------------------------------------------------------------------
// Manifest impl
// ---------------------------------------------------------------------------

impl Manifest {
    /// Create a fresh manifest for a new corpus.
    pub fn new() -> Self {
        Self {
            version: 0,
            schema_version: schema::SCHEMA_VERSION.to_string(),
            segments: Vec::new(),
            active_segment: None,
        }
    }

    /// Resolve the manifest file path within a `.corpus/` directory.
    pub fn file_path(corpus_dir: &Path) -> PathBuf {
        corpus_dir.join(MANIFEST_FILE)
    }

    /// Load a manifest from the `.corpus/` directory.
    pub fn load(corpus_dir: &Path) -> Result<Self> {
        let path = Self::file_path(corpus_dir);
        let data = std::fs::read_to_string(&path).map_err(|e| {
            Error::ManifestLoad(path.clone(), e.to_string())
        })?;
        let manifest: Manifest =
            serde_json::from_str(&data).map_err(|e| Error::ManifestLoad(path, e.to_string()))?;
        Ok(manifest)
    }

    /// Atomically save the manifest to the `.corpus/` directory.
    ///
    /// Writes to a temporary file then renames — so a crash during save
    /// leaves the previous manifest intact.
    pub fn save(&self, corpus_dir: &Path) -> Result<()> {
        let path = Self::file_path(corpus_dir);
        let tmp_path = path.with_extension("json.tmp");

        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&tmp_path, data.as_bytes())
            .map_err(|e| Error::ManifestSave(tmp_path.clone(), e.to_string()))?;
        std::fs::rename(&tmp_path, &path)
            .map_err(|e| Error::ManifestSave(path, e.to_string()))?;

        Ok(())
    }

    /// Bump the manifest version.
    pub fn bump_version(&mut self) {
        self.version += 1;
    }

    /// Add a new segment entry.
    pub fn add_segment(&mut self, meta: SegmentMeta) {
        self.segments.push(meta);
    }

    /// Mark a segment as sealed: set its hash, size, chunk_count, and sealed_at.
    pub fn seal_segment(
        &mut self,
        id: SegmentId,
        hash: &str,
        size: u64,
        chunk_count: u64,
        sealed_at: &str,
    ) {
        if let Some(seg) = self.segments.iter_mut().find(|s| s.id == id) {
            seg.hash = Some(hash.to_string());
            seg.size = size;
            seg.chunk_count = chunk_count;
            seg.sealed_at = Some(sealed_at.to_string());
        }
    }

    /// The next segment ID (max existing + 1, or 1 if none).
    pub fn next_segment_id(&self) -> SegmentId {
        let max = self.segments.iter().map(|s| s.id.0).max().unwrap_or(0);
        SegmentId(max + 1)
    }

    /// Resolve paths for all segments within a segments directory.
    pub fn all_segment_paths(&self, segments_dir: &Path) -> Vec<PathBuf> {
        self.segments
            .iter()
            .map(|s| s.id.path_in(segments_dir))
            .collect()
    }

    /// Compute the diff between this manifest and another (e.g. a remote one).
    ///
    /// `self` is the local manifest; `other` is the remote manifest.
    pub fn diff(&self, other: &Manifest) -> ManifestDiff {
        let remote_set: HashSet<(u64, Option<&str>)> = other
            .segments
            .iter()
            .map(|s| (s.id.0, s.hash.as_deref()))
            .collect();

        let added: Vec<SegmentMeta> = self
            .segments
            .iter()
            .filter(|s| !remote_set.contains(&(s.id.0, s.hash.as_deref())))
            .cloned()
            .collect();

        let remote_ids: HashSet<u64> = other.segments.iter().map(|s| s.id.0).collect();
        let local_ids: HashSet<u64> = self.segments.iter().map(|s| s.id.0).collect();
        let removed: Vec<SegmentId> = remote_ids
            .difference(&local_ids)
            .map(|&id| SegmentId(id))
            .collect();

        let active_changed = self.active_segment != other.active_segment;

        ManifestDiff {
            added,
            removed,
            active_changed,
        }
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_new_and_roundtrip() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dir = tmp.path();

        let mut m = Manifest::new();
        m.add_segment(SegmentMeta::active(SegmentId(1)));
        m.active_segment = Some(SegmentId(1));
        m.bump_version();

        m.save(dir).expect("save");
        let loaded = Manifest::load(dir).expect("load");

        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.segments.len(), 1);
        assert_eq!(loaded.active_segment, Some(SegmentId(1)));
    }

    #[test]
    fn manifest_diff_detects_added_and_removed() {
        let mut local = Manifest::new();
        local.add_segment(SegmentMeta {
            id: SegmentId(1),
            hash: Some("aaa".into()),
            size: 100,
            chunk_count: 10,
            sealed_at: Some("2026-01-01T00:00:00Z".into()),
        });
        local.add_segment(SegmentMeta {
            id: SegmentId(2),
            hash: Some("bbb".into()),
            size: 200,
            chunk_count: 20,
            sealed_at: Some("2026-01-02T00:00:00Z".into()),
        });

        let mut remote = Manifest::new();
        remote.add_segment(SegmentMeta {
            id: SegmentId(1),
            hash: Some("aaa".into()),
            size: 100,
            chunk_count: 10,
            sealed_at: Some("2026-01-01T00:00:00Z".into()),
        });
        remote.add_segment(SegmentMeta {
            id: SegmentId(3),
            hash: Some("ccc".into()),
            size: 300,
            chunk_count: 30,
            sealed_at: Some("2026-01-03T00:00:00Z".into()),
        });

        let diff = local.diff(&remote);

        // Segment 2 is in local but not remote → added
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].id, SegmentId(2));

        // Segment 3 is in remote but not local → removed
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0], SegmentId(3));
    }

    #[test]
    fn next_segment_id_increments() {
        let mut m = Manifest::new();
        assert_eq!(m.next_segment_id(), SegmentId(1));

        m.add_segment(SegmentMeta::active(SegmentId(1)));
        assert_eq!(m.next_segment_id(), SegmentId(2));

        m.add_segment(SegmentMeta::active(SegmentId(5)));
        assert_eq!(m.next_segment_id(), SegmentId(6));
    }
}
