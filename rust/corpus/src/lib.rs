//! Stencila Corpus: segmented SQLite search DB builder and query engine.
//!
//! This crate constructs and queries a local segmented SQLite search DB from
//! a corpus of documents (any format supported by Stencila). The output is a
//! set of immutable SQLite segment files plus a small manifest and state DB,
//! designed for efficient sync and portability.
//!
//! # Architecture
//!
//! ```text
//! <document-root>/
//!   .corpus/
//!     manifest.json        ← segment registry + schema version
//!     state.sqlite         ← doc routing, tombstones, build metadata
//!     segments/
//!       seg_000001.sqlite  ← sealed, immutable, content-addressable
//!       seg_000002.sqlite  ← active (read-write)
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use stencila_corpus::{Corpus, BuildConfig, CorpusQuery};
//!
//! # async fn example() -> eyre::Result<()> {
//! let mut corpus = Corpus::open("path/to/documents")?;
//! let report = corpus.build(BuildConfig::default()).await?;
//! println!("Ingested {} files, wrote {} chunks", report.files_ingested, report.chunks_written);
//!
//! let result = corpus.query(&CorpusQuery {
//!     text: "protein folding".into(),
//!     node_types: vec![],
//!     limit: 10,
//! }).await?;
//!
//! for hit in &result.hits {
//!     println!("{}: {} (score: {:.3})", hit.doc_id, hit.text, hit.score);
//! }
//! # Ok(())
//! # }
//! ```

pub mod builder;
pub mod error;
pub mod manifest;
pub mod query;
pub mod schema;
pub mod segment;
pub mod state;
pub mod sync;

use std::path::{Path, PathBuf};

pub use builder::{BuildConfig, BuildReport, Builder};
pub use manifest::{Manifest, ManifestDiff, SegmentMeta};
pub use query::{CorpusQuery, QueryEngine, QueryResult};
pub use segment::{ChunkRow, QueryHit, Segment, SegmentId};
pub use state::State;
pub use sync::{SyncClient, SyncReport};

use error::Result;

// ---------------------------------------------------------------------------
// CorpusStats
// ---------------------------------------------------------------------------

/// Summary statistics for a corpus.
#[derive(Debug, Clone)]
pub struct CorpusStats {
    pub total_segments: usize,
    pub sealed_segments: usize,
    pub total_chunks: u64,
    pub total_size_bytes: u64,
    pub schema_version: String,
    pub manifest_version: u64,
    pub has_active_segment: bool,
}

// ---------------------------------------------------------------------------
// Corpus — the main facade
// ---------------------------------------------------------------------------

/// High-level facade for building, querying, and managing a corpus.
pub struct Corpus {
    /// The user's document root.
    root: PathBuf,
    /// The `.corpus/` directory inside the root.
    corpus_dir: PathBuf,
    /// The `segments/` directory inside `.corpus/`.
    segments_dir: PathBuf,
    /// The manifest.
    manifest: Manifest,
}

impl Corpus {
    /// Open an existing corpus or initialize a new one at `root`.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let corpus_dir = root.join(".corpus");
        let segments_dir = corpus_dir.join("segments");

        std::fs::create_dir_all(&segments_dir)?;

        let manifest = if Manifest::file_path(&corpus_dir).exists() {
            Manifest::load(&corpus_dir)?
        } else {
            let m = Manifest::new();
            m.save(&corpus_dir)?;
            m
        };

        Ok(Self {
            root,
            corpus_dir,
            segments_dir,
            manifest,
        })
    }

    /// Build or update the corpus index from the document directory.
    pub async fn build(&mut self, config: BuildConfig) -> Result<BuildReport> {
        let state = State::open(&self.corpus_dir)?;
        let builder = Builder::new(&self.corpus_dir, &self.root, config);
        let report = builder.build(&mut self.manifest, &state).await?;

        // Reload manifest after build (it was saved by the builder).
        self.manifest = Manifest::load(&self.corpus_dir)?;

        Ok(report)
    }

    /// Execute a query across all segments.
    pub async fn query(&self, query: &CorpusQuery) -> Result<QueryResult> {
        let state = State::open(&self.corpus_dir)?;
        let engine = QueryEngine::from_parts(
            self.segments_dir.clone(),
            self.manifest.clone(),
            state,
        );
        engine.search(query).await
    }

    /// Seal the active segment (e.g. before sync).
    pub fn seal(&mut self) -> Result<()> {
        if let Some(active_id) = self.manifest.active_segment {
            let mut segment =
                Segment::create_or_reopen(&self.segments_dir, active_id)?;
            let sealed = segment.seal()?;
            let count = segment.chunk_count()?;
            self.manifest.seal_segment(
                sealed.id,
                &sealed.hash,
                sealed.size,
                count,
                &sealed.sealed_at,
            );
            self.manifest.active_segment = None;
            self.manifest.bump_version();
            self.manifest.save(&self.corpus_dir)?;
        }
        Ok(())
    }

    /// Compute the diff between the local manifest and a remote manifest.
    pub fn diff(&self, remote: &Manifest) -> ManifestDiff {
        self.manifest.diff(remote)
    }

    /// List all segments with metadata.
    pub fn segments(&self) -> &[SegmentMeta] {
        &self.manifest.segments
    }

    /// Get a reference to the current manifest.
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Corpus statistics.
    pub fn stats(&self) -> CorpusStats {
        let total_chunks: u64 = self.manifest.segments.iter().map(|s| s.chunk_count).sum();
        let total_size: u64 = self.manifest.segments.iter().map(|s| s.size).sum();
        let sealed = self
            .manifest
            .segments
            .iter()
            .filter(|s| s.sealed_at.is_some())
            .count();

        CorpusStats {
            total_segments: self.manifest.segments.len(),
            sealed_segments: sealed,
            total_chunks,
            total_size_bytes: total_size,
            schema_version: self.manifest.schema_version.clone(),
            manifest_version: self.manifest.version,
            has_active_segment: self.manifest.active_segment.is_some(),
        }
    }

    /// Push the corpus to Stencila Cloud.
    ///
    /// Seals the active segment first, then uploads any segments the server
    /// doesn't already have, and stores the manifest.
    pub async fn push(
        &mut self,
        workspace_id: &str,
        corpus_id: &str,
        manifest_name: &str,
    ) -> Result<SyncReport> {
        // Seal before push so everything is content-addressable.
        self.seal()?;

        let client = SyncClient::new(workspace_id, corpus_id, manifest_name)
            .await
            .map_err(|e| error::Error::Other(format!("failed to create sync client: {e}")))?;

        client.push(&self.corpus_dir, &self.manifest).await
    }

    /// Pull the corpus from Stencila Cloud.
    ///
    /// Downloads any missing segments from the server and replaces the
    /// local manifest with the remote one.
    pub async fn pull(
        &mut self,
        workspace_id: &str,
        corpus_id: &str,
        manifest_name: &str,
    ) -> Result<SyncReport> {
        let client = SyncClient::new(workspace_id, corpus_id, manifest_name)
            .await
            .map_err(|e| error::Error::Other(format!("failed to create sync client: {e}")))?;

        let (remote_manifest, report) = client.pull(&self.corpus_dir).await?;
        self.manifest = remote_manifest;
        Ok(report)
    }

    /// Path to the `.corpus/` directory.
    pub fn corpus_dir(&self) -> &Path {
        &self.corpus_dir
    }

    /// Path to the document root.
    pub fn root(&self) -> &Path {
        &self.root
    }
}
