//! Corpus build orchestrator.
//!
//! Scans a document directory, detects changes against the state DB,
//! tombstones removed/changed documents, ingests new ones into the active
//! segment, and handles rollover when the segment gets too large.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use stencila_codec_text_trait::to_text;
use stencila_format::Format;
use tracing;

use crate::error::{Error, Result};
use crate::manifest::{Manifest, SegmentMeta};
use crate::segment::{ChunkRow, Segment, SegmentId};
use crate::state::State;

// ---------------------------------------------------------------------------
// BuildConfig
// ---------------------------------------------------------------------------

/// Configuration for a corpus build.
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Maximum chunks per segment before rollover.
    pub max_chunks_per_segment: u64,
    /// Maximum segment file size in bytes before rollover.
    pub max_segment_bytes: u64,
    /// Glob patterns to exclude from scanning.
    pub exclude_patterns: Vec<String>,
    /// If true, skip unchanged files (default: true).
    pub incremental: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            max_chunks_per_segment: 100_000,
            max_segment_bytes: 512 * 1024 * 1024, // 512 MB
            exclude_patterns: vec![
                ".corpus/**".into(),
                ".git/**".into(),
                ".stencila/**".into(),
                "node_modules/**".into(),
            ],
            incremental: true,
        }
    }
}

// ---------------------------------------------------------------------------
// BuildReport
// ---------------------------------------------------------------------------

/// Summary of a build run.
#[derive(Debug, Clone)]
pub struct BuildReport {
    pub files_scanned: usize,
    pub files_ingested: usize,
    pub files_unchanged: usize,
    pub files_tombstoned: usize,
    pub files_failed: Vec<(PathBuf, String)>,
    pub chunks_written: usize,
    pub segments_sealed: usize,
    pub duration: std::time::Duration,
}

// ---------------------------------------------------------------------------
// ScannedFile
// ---------------------------------------------------------------------------

/// A file discovered during directory scanning.
#[derive(Debug, Clone)]
struct ScannedFile {
    /// Absolute path to the file.
    path: PathBuf,
    /// Relative path from the corpus root (used as the stored path).
    rel_path: PathBuf,
    /// blake3 hash of the file contents.
    hash: String,
}

// ---------------------------------------------------------------------------
// BuildPlan
// ---------------------------------------------------------------------------

/// The plan computed by diffing scanned files against the state.
struct BuildPlan {
    to_ingest: Vec<ScannedFile>,
    to_tombstone: Vec<(String, SegmentId)>, // (doc_id, segment_id)
    unchanged: usize,
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// The corpus build orchestrator.
pub struct Builder {
    corpus_dir: PathBuf, // .corpus/ directory
    segments_dir: PathBuf,
    root: PathBuf, // document root
    config: BuildConfig,
}

impl Builder {
    /// Create a builder for the given document root.
    ///
    /// `corpus_dir` is the `.corpus/` directory; `root` is the document directory.
    pub fn new(corpus_dir: &Path, root: &Path, config: BuildConfig) -> Self {
        let segments_dir = corpus_dir.join("segments");
        Self {
            corpus_dir: corpus_dir.to_path_buf(),
            segments_dir,
            root: root.to_path_buf(),
            config,
        }
    }

    /// Run the full build pipeline.
    pub async fn build(
        &self,
        manifest: &mut Manifest,
        state: &State,
    ) -> Result<BuildReport> {
        let start = Instant::now();

        // Ensure segments directory exists.
        std::fs::create_dir_all(&self.segments_dir)?;

        // 1. Scan
        let scanned = self.scan_directory()?;
        let files_scanned = scanned.len();

        // 2. Diff
        let plan = self.compute_plan(&scanned, state)?;
        let files_unchanged = plan.unchanged;
        let files_tombstoned = plan.to_tombstone.len();

        // 3. Tombstone
        for (doc_id, seg_id) in &plan.to_tombstone {
            state.tombstone_doc(doc_id, *seg_id)?;
        }

        // 4. Ensure we have an active segment
        let mut segment = self.ensure_active_segment(manifest)?;

        // 5. Ingest
        let mut files_ingested = 0;
        let mut chunks_written = 0;
        let mut segments_sealed = 0;
        let mut files_failed = Vec::new();

        for file in &plan.to_ingest {
            match self.ingest_file(file, &mut segment, state).await {
                Ok(n_chunks) => {
                    files_ingested += 1;
                    chunks_written += n_chunks;

                    // 5b. Rollover check
                    let count = segment.chunk_count()?;
                    let size = segment.file_size()?;
                    if count >= self.config.max_chunks_per_segment
                        || size >= self.config.max_segment_bytes
                    {
                        let sealed = segment.seal()?;
                        manifest.seal_segment(
                            sealed.id,
                            &sealed.hash,
                            sealed.size,
                            count,
                            &sealed.sealed_at,
                        );
                        segments_sealed += 1;

                        let new_id = manifest.next_segment_id();
                        segment = Segment::create(&self.segments_dir, new_id)?;
                        manifest.add_segment(SegmentMeta::active(new_id));
                        manifest.active_segment = Some(new_id);
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        path = %file.rel_path.display(),
                        error = %e,
                        "failed to ingest file, skipping"
                    );
                    files_failed.push((file.rel_path.clone(), e.to_string()));
                }
            }
        }

        // 6. Update manifest
        // Update the active segment's stats.
        if let Some(active_id) = manifest.active_segment {
            if let Some(meta) = manifest.segments.iter_mut().find(|s| s.id == active_id) {
                meta.chunk_count = segment.chunk_count()?;
                meta.size = segment.file_size()?;
            }
        }
        manifest.bump_version();
        manifest.save(&self.corpus_dir)?;

        state.set_meta("last_build_at", &chrono::Utc::now().to_rfc3339())?;

        Ok(BuildReport {
            files_scanned,
            files_ingested,
            files_unchanged,
            files_tombstoned,
            files_failed,
            chunks_written,
            segments_sealed,
            duration: start.elapsed(),
        })
    }

    // -- Scanning ----------------------------------------------------------

    fn scan_directory(&self) -> Result<Vec<ScannedFile>> {
        let exclude_patterns: Vec<glob::Pattern> = self
            .config
            .exclude_patterns
            .iter()
            .filter_map(|p| glob::Pattern::new(p).ok())
            .collect();

        let mut files = Vec::new();

        for entry in walkdir::WalkDir::new(&self.root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let abs_path = entry.path();
            let rel_path = match abs_path.strip_prefix(&self.root) {
                Ok(r) => r.to_path_buf(),
                Err(_) => continue,
            };

            // Check excludes.
            let rel_str = rel_path.to_string_lossy();
            if exclude_patterns.iter().any(|p| p.matches(&rel_str)) {
                continue;
            }

            // Check if format is supported for decoding.
            let format = Format::from_path(abs_path);
            if matches!(format, Format::Unknown) {
                continue;
            }
            if !stencila_codecs::from_path_is_supported(abs_path) {
                continue;
            }

            // Hash the file.
            let data = match std::fs::read(abs_path) {
                Ok(d) => d,
                Err(e) => {
                    tracing::warn!(path = %abs_path.display(), error = %e, "could not read file");
                    continue;
                }
            };
            let hash = blake3::hash(&data).to_hex().to_string();

            files.push(ScannedFile {
                path: abs_path.to_path_buf(),
                rel_path,
                hash,
            });
        }

        Ok(files)
    }

    // -- Change detection --------------------------------------------------

    fn compute_plan(&self, scanned: &[ScannedFile], state: &State) -> Result<BuildPlan> {
        let file_index = state.file_index()?;

        let mut to_ingest = Vec::new();
        let mut to_tombstone = Vec::new();
        let mut unchanged = 0;

        // Build a set of scanned paths for "removed file" detection.
        let scanned_paths: HashMap<&Path, &ScannedFile> =
            scanned.iter().map(|f| (f.rel_path.as_path(), f)).collect();

        for file in scanned {
            match file_index.get(&file.rel_path) {
                Some(entry) if entry.file_hash == file.hash && self.config.incremental => {
                    // Unchanged
                    unchanged += 1;
                }
                Some(entry) => {
                    // Changed: tombstone old, re-ingest
                    to_tombstone.push((entry.doc_id.clone(), entry.segment_id));
                    to_ingest.push(file.clone());
                }
                None => {
                    // New file
                    to_ingest.push(file.clone());
                }
            }
        }

        // Detect removed files (in state but not on disk).
        for (path, entry) in &file_index {
            if !scanned_paths.contains_key(path.as_path()) {
                to_tombstone.push((entry.doc_id.clone(), entry.segment_id));
            }
        }

        Ok(BuildPlan {
            to_ingest,
            to_tombstone,
            unchanged,
        })
    }

    // -- Segment management ------------------------------------------------

    fn ensure_active_segment(&self, manifest: &mut Manifest) -> Result<Segment> {
        if let Some(active_id) = manifest.active_segment {
            let path = active_id.path_in(&self.segments_dir);
            if path.exists() {
                // Re-open the existing active segment for writing.
                let conn = rusqlite::Connection::open(&path)?;
                // This is a bit of a hack to reopen — we reconstruct.
                // In practice the segment was left open; for now we reopen.
                drop(conn);
                return Segment::create_or_reopen(&self.segments_dir, active_id);
            }
        }

        // No active segment — create one.
        let id = manifest.next_segment_id();
        let segment = Segment::create(&self.segments_dir, id)?;
        manifest.add_segment(SegmentMeta::active(id));
        manifest.active_segment = Some(id);
        Ok(segment)
    }

    // -- Ingestion ---------------------------------------------------------

    async fn ingest_file(
        &self,
        file: &ScannedFile,
        segment: &mut Segment,
        state: &State,
    ) -> Result<usize> {
        // Decode file into a Stencila Node.
        let node = stencila_codecs::from_path(&file.path, None)
            .await
            .map_err(|e| Error::Other(format!("decode {}: {e}", file.rel_path.display())))?;

        // Extract chunks. For now: one chunk per document with full text.
        let text = to_text(&node);
        let doc_id = doc_id_for(&file.rel_path);
        let chunk_id = format!("{doc_id}_0");

        let chunks = vec![ChunkRow {
            chunk_id,
            doc_id: doc_id.clone(),
            path: file.rel_path.to_string_lossy().to_string(),
            node_type: node_type_name(&node),
            text,
            metadata: "{}".into(),
        }];

        let n = chunks.len();
        segment.write_batch(&chunks)?;
        segment.register_doc(
            &doc_id,
            &file.rel_path.to_string_lossy(),
            &file.hash,
        )?;
        state.register_doc(&doc_id, &file.rel_path, &file.hash, segment.id())?;

        Ok(n)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Deterministic doc ID from relative path.
fn doc_id_for(rel_path: &Path) -> String {
    let hash = blake3::hash(rel_path.to_string_lossy().as_bytes());
    format!("doc_{}", &hash.to_hex()[..16])
}

/// Get the top-level node type name as a string.
fn node_type_name(node: &stencila_schema::Node) -> String {
    // The Node enum's Display impl gives us the variant name.
    format!("{node}")
}

// ---------------------------------------------------------------------------
// Segment::create_or_reopen helper
// ---------------------------------------------------------------------------

impl Segment {
    /// Open an existing active segment (not sealed) or create a new one.
    ///
    /// If the segment file exists, opens it read-write without re-initializing
    /// the schema. If not, creates fresh.
    pub fn create_or_reopen(segments_dir: &Path, id: SegmentId) -> Result<Segment> {
        let path = id.path_in(segments_dir);
        if path.exists() {
            let conn = rusqlite::Connection::open(&path)?;
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
            Ok(Segment {
                id,
                path,
                conn,
                sealed: false,
            })
        } else {
            Segment::create(segments_dir, id)
        }
    }
}
