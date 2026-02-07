//! Parallel query engine.
//!
//! Fans out queries across all segments with bounded parallelism,
//! collects results, filters tombstones, and merges into a top-k result set.

use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::error::{Error, Result};
use crate::manifest::Manifest;
use crate::segment::{QueryHit, Segment};
use crate::state::State;

// ---------------------------------------------------------------------------
// CorpusQuery
// ---------------------------------------------------------------------------

/// A query to execute across the corpus.
///
/// Schema-specific query parameters (citation constraints, etc.) will be
/// added later. For now this carries the minimum: text + node_type filters.
#[derive(Debug, Clone)]
pub struct CorpusQuery {
    /// Full-text search query string (FTS5 syntax).
    pub text: String,
    /// Optional node type filters. Empty means "all types".
    pub node_types: Vec<String>,
    /// Maximum results to return.
    pub limit: usize,
}

// ---------------------------------------------------------------------------
// QueryResult
// ---------------------------------------------------------------------------

/// Merged, tombstone-filtered query results.
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub hits: Vec<QueryHit>,
    pub segments_searched: usize,
    pub total_hits_before_merge: usize,
}

// ---------------------------------------------------------------------------
// QueryEngine
// ---------------------------------------------------------------------------

/// Engine that executes queries across all segments in parallel.
pub struct QueryEngine {
    segments_dir: PathBuf,
    manifest: Manifest,
    state: State,
    parallelism: usize,
}

impl QueryEngine {
    /// Create a new query engine.
    ///
    /// `corpus_dir` is the `.corpus/` directory.
    pub fn new(corpus_dir: &Path) -> Result<Self> {
        let manifest = Manifest::load(corpus_dir)?;
        let state = State::open(corpus_dir)?;
        let segments_dir = corpus_dir.join("segments");

        Ok(Self {
            segments_dir,
            manifest,
            state,
            parallelism: 8,
        })
    }

    /// Create from pre-loaded components (for use in the Corpus facade).
    pub fn from_parts(
        segments_dir: PathBuf,
        manifest: Manifest,
        state: State,
    ) -> Self {
        Self {
            segments_dir,
            manifest,
            state,
            parallelism: 8,
        }
    }

    /// Set maximum parallelism for segment queries.
    pub fn with_parallelism(mut self, n: usize) -> Self {
        self.parallelism = n.max(1);
        self
    }

    /// Execute a full-text search across all segments.
    pub async fn search(&self, query: &CorpusQuery) -> Result<QueryResult> {
        // 1. Load tombstone set.
        let tombstones = self.state.all_tombstoned_doc_ids()?;

        // 2. Collect segment paths.
        let segment_paths = self.manifest.all_segment_paths(&self.segments_dir);
        let segments_searched = segment_paths.len();

        if segment_paths.is_empty() {
            return Ok(QueryResult {
                hits: Vec::new(),
                segments_searched: 0,
                total_hits_before_merge: 0,
            });
        }

        // 3. Fan out with bounded parallelism.
        let semaphore = Arc::new(Semaphore::new(self.parallelism));
        let mut handles = Vec::with_capacity(segment_paths.len());

        for path in segment_paths {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| Error::Other(format!("semaphore acquire failed: {e}")))?;
            let query_text = query.text.clone();
            let node_types = query.node_types.clone();
            let limit = query.limit;

            handles.push(tokio::task::spawn_blocking(move || {
                let result = search_segment(&path, &query_text, &node_types, limit);
                drop(permit);
                result
            }));
        }

        // 4. Collect results.
        let mut all_hits = Vec::new();
        let mut total_before_merge = 0;

        for handle in handles {
            match handle.await {
                Ok(Ok(hits)) => {
                    total_before_merge += hits.len();
                    all_hits.extend(hits);
                }
                Ok(Err(e)) => {
                    tracing::warn!(error = %e, "segment query failed, skipping");
                }
                Err(e) => {
                    tracing::warn!(error = %e, "segment query task panicked, skipping");
                }
            }
        }

        // 5. Filter tombstones.
        filter_tombstones(&mut all_hits, &tombstones);

        // 6. Sort by score (descending) and truncate.
        all_hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
        });
        all_hits.truncate(query.limit);

        Ok(QueryResult {
            hits: all_hits,
            segments_searched,
            total_hits_before_merge: total_before_merge,
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers (run inside spawn_blocking)
// ---------------------------------------------------------------------------

/// Search a single segment file. Called inside `spawn_blocking`.
fn search_segment(
    path: &Path,
    query_text: &str,
    node_types: &[String],
    limit: usize,
) -> Result<Vec<QueryHit>> {
    let segment = Segment::open_readonly(path)?;
    segment.fts_search(query_text, node_types, limit)
}

/// Remove hits whose `doc_id` is in the tombstone set.
fn filter_tombstones(hits: &mut Vec<QueryHit>, tombstones: &HashSet<String>) {
    if tombstones.is_empty() {
        return;
    }
    hits.retain(|hit| !tombstones.contains(&hit.doc_id));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment::{ChunkRow, Segment, SegmentId};

    /// Build a minimal two-segment corpus and query across both.
    #[tokio::test]
    async fn parallel_query_across_segments() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let corpus_dir = tmp.path();
        let segments_dir = corpus_dir.join("segments");
        std::fs::create_dir_all(&segments_dir).expect("mkdir segments");

        // Segment 1
        let mut seg1 = Segment::create(&segments_dir, SegmentId(1)).expect("seg1");
        seg1.write_batch(&[ChunkRow {
            chunk_id: "c1".into(),
            doc_id: "d1".into(),
            path: "a.md".into(),
            node_type: "Paragraph".into(),
            text: "Rust is a systems programming language".into(),
            metadata: "{}".into(),
        }])
        .expect("write seg1");
        let sealed1 = seg1.seal().expect("seal seg1");

        // Segment 2
        let mut seg2 = Segment::create(&segments_dir, SegmentId(2)).expect("seg2");
        seg2.write_batch(&[ChunkRow {
            chunk_id: "c2".into(),
            doc_id: "d2".into(),
            path: "b.md".into(),
            node_type: "Paragraph".into(),
            text: "Rust programming enables memory safety without garbage collection".into(),
            metadata: "{}".into(),
        }])
        .expect("write seg2");
        let sealed2 = seg2.seal().expect("seal seg2");

        // Build manifest
        let mut manifest = Manifest::new();
        manifest.add_segment(crate::manifest::SegmentMeta {
            id: SegmentId(1),
            hash: Some(sealed1.hash),
            size: sealed1.size,
            chunk_count: 1,
            sealed_at: Some(sealed1.sealed_at),
        });
        manifest.add_segment(crate::manifest::SegmentMeta {
            id: SegmentId(2),
            hash: Some(sealed2.hash),
            size: sealed2.size,
            chunk_count: 1,
            sealed_at: Some(sealed2.sealed_at),
        });
        manifest.save(corpus_dir).expect("save manifest");

        // State (no tombstones)
        let state = State::open(corpus_dir).expect("state");

        let engine = QueryEngine::from_parts(
            segments_dir,
            manifest,
            state,
        );

        // Query for "Rust programming" — should match both segments
        let result = engine
            .search(&CorpusQuery {
                text: "rust programming".into(),
                node_types: vec![],
                limit: 10,
            })
            .await
            .expect("search");

        assert_eq!(result.segments_searched, 2);
        assert_eq!(result.hits.len(), 2);
    }

    #[tokio::test]
    async fn tombstones_filter_results() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let corpus_dir = tmp.path();
        let segments_dir = corpus_dir.join("segments");
        std::fs::create_dir_all(&segments_dir).expect("mkdir segments");

        let mut seg = Segment::create(&segments_dir, SegmentId(1)).expect("seg");
        seg.write_batch(&[
            ChunkRow {
                chunk_id: "c1".into(),
                doc_id: "d_alive".into(),
                path: "alive.md".into(),
                node_type: "Paragraph".into(),
                text: "This document is alive and well".into(),
                metadata: "{}".into(),
            },
            ChunkRow {
                chunk_id: "c2".into(),
                doc_id: "d_dead".into(),
                path: "dead.md".into(),
                node_type: "Paragraph".into(),
                text: "This document is alive but will be tombstoned".into(),
                metadata: "{}".into(),
            },
        ])
        .expect("write");
        let sealed = seg.seal().expect("seal");

        let mut manifest = Manifest::new();
        manifest.add_segment(crate::manifest::SegmentMeta {
            id: SegmentId(1),
            hash: Some(sealed.hash),
            size: sealed.size,
            chunk_count: 2,
            sealed_at: Some(sealed.sealed_at),
        });
        manifest.save(corpus_dir).expect("save manifest");

        // Create state with a tombstone for d_dead
        let state = State::open(corpus_dir).expect("state");
        state
            .tombstone_doc("d_dead", SegmentId(1))
            .expect("tombstone");

        let engine = QueryEngine::from_parts(segments_dir, manifest, state);

        let result = engine
            .search(&CorpusQuery {
                text: "alive".into(),
                node_types: vec![],
                limit: 10,
            })
            .await
            .expect("search");

        // Both chunks match "alive" but d_dead is tombstoned
        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.hits[0].doc_id, "d_alive");
    }
}
