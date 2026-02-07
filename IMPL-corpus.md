# Implementation Plan: `rust/corpus` — Segmented DB Engine

**Scope**: segment lifecycle, build pipeline, parallel query, cloud-sync readiness.
Schema internals (tables, FTS config, chunk fields) are **out of scope** — they will be iterated separately.

---

## 1. Crate Scaffold & Workspace Integration

### 1.1 Create `rust/corpus/`

```
rust/corpus/
  Cargo.toml
  src/
    lib.rs          — public API re-exports
    segment.rs      — segment file abstraction
    manifest.rs     — manifest read/write
    state.rs        — mutable state DB
    builder.rs      — corpus build orchestrator
    query.rs        — parallel query engine
    schema.rs       — segment schema init (placeholder for now)
    error.rs        — crate error types
```

### 1.2 Workspace `Cargo.toml`

Add `rust/corpus` to the workspace `members`. Key dependencies:

| Dependency | Source | Purpose |
|---|---|---|
| `rusqlite` | crates.io | SQLite segment read/write (with `bundled`, `fts5` features) |
| `blake3` | crates.io | Content-addressable hashing (files + segments) |
| `tokio` | workspace | Async runtime for parallel query |
| `serde` / `serde_json` | workspace | Manifest serialization |
| `eyre` | workspace | Error handling |
| `stencila-schema` | workspace | `Node`, `NodeType`, `WalkNode`, `Visitor` |
| `stencila-codecs` | workspace | `codecs::from_path()` — decode any supported format |
| `stencila-format` | workspace | `Format::from_path()` — detect file type |

`rusqlite` is chosen over Kuzu because segments are append-only tabular stores (not graphs), and SQLite gives us: single-file immutability, zero-daemon portability, content-addressable hashing of sealed files, and trivial cloud upload as opaque blobs.

### 1.3 Feature flags

```toml
[features]
default = ["fts"]
fts = []           # FTS5 virtual tables in segments (on by default)
embeddings = []    # vector column + similarity search (off by default)
```

---

## 2. Segment Abstraction (`segment.rs`)

A **segment** is a single, self-contained SQLite file. Once **sealed**, it is never modified.

### 2.1 Segment identity

```rust
/// Opaque segment identifier. Monotonically increasing per corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SegmentId(u64);
```

Segment filenames: `seg_{id:06}.sqlite` (e.g. `seg_000001.sqlite`).

### 2.2 Segment lifecycle

```
  ┌──────────┐   seal()   ┌──────────┐
  │  Active   │ ────────►  │  Sealed  │
  │ (RW)     │            │ (RO)     │
  └──────────┘            └──────────┘
       │                       │
       │  write_chunks()       │  open_readonly()
       │  init_schema()        │  query()
       │                       │  blake3 hash (stable)
```

**Active segment**: exactly one at a time. Opened read-write. Receives new chunks from the builder. Sealed when a rollover threshold is hit.

**Sealed segment**: read-only. Identified by its blake3 hash (computed at seal time). Never modified again.

### 2.3 Core type

```rust
pub struct Segment {
    id: SegmentId,
    conn: rusqlite::Connection,  // RW for active, RO for sealed
    sealed: bool,
}
```

Key methods:

| Method | Description |
|---|---|
| `Segment::create(dir, id)` | Create new active segment, run schema init |
| `Segment::open_readonly(path)` | Open sealed segment for queries |
| `Segment::init_schema(&self)` | Apply the current schema SQL (delegated to `schema.rs`) |
| `Segment::write_batch(&self, rows)` | Batch-insert rows into the segment (active only) |
| `Segment::seal(&mut self) → SegmentMeta` | Finalize: `VACUUM`, set `PRAGMA journal_mode=OFF`, compute blake3 hash, mark RO |
| `Segment::chunk_count(&self) → u64` | Row count for rollover decisions |
| `Segment::file_size(&self) → u64` | Byte size for rollover decisions |
| `Segment::hash(path) → blake3::Hash` | Static: hash a sealed segment file |

### 2.4 Rollover policy

The builder seals the active segment and opens a new one when **any** threshold is exceeded:

| Threshold | Default | Configurable |
|---|---|---|
| Chunk count | 100,000 | Yes |
| File size | 512 MB | Yes |

These are intentionally generous — most user corpora will fit in a single segment. The segmented design pays off at scale and for sync.

### 2.5 Schema initialization (`schema.rs`)

`schema.rs` exposes a single function:

```rust
pub fn init(conn: &rusqlite::Connection) -> Result<()>
```

This runs the DDL that creates tables, FTS virtual tables, and indexes inside the segment. **The actual DDL is out of scope for this plan** and will be developed iteratively. For now, `init` can create a minimal placeholder schema sufficient to test the segment lifecycle.

---

## 3. Manifest (`manifest.rs`)

The manifest is the **source of truth** for which segments exist and their integrity.

### 3.1 Schema

```rust
#[derive(Serialize, Deserialize)]
pub struct Manifest {
    /// Monotonically increasing. Bumped on every build that modifies segments.
    pub version: u64,

    /// Schema version applied to all segments. All segments in a corpus
    /// share the same schema version (migration = full rebuild).
    pub schema_version: String,

    /// Ordered list of segments (oldest first).
    pub segments: Vec<SegmentMeta>,

    /// ID of the currently active (unsealed) segment, if any.
    /// None when the corpus is fully sealed (e.g. after explicit `corpus seal`).
    pub active_segment: Option<SegmentId>,

    /// blake3 hash of this manifest file's content (excluding this field).
    /// Used by sync to detect manifest changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_hash: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SegmentMeta {
    pub id: SegmentId,
    /// blake3 hash of the sealed segment file. None for the active segment.
    pub hash: Option<String>,
    /// Byte size of the segment file.
    pub size: u64,
    /// Number of chunks in this segment.
    pub chunk_count: u64,
    /// Timestamp when the segment was sealed. None for active segment.
    pub sealed_at: Option<String>,  // RFC 3339
}
```

### 3.2 File location

```
<corpus-root>/
  .corpus/
    manifest.json
    state.sqlite
    segments/
      seg_000001.sqlite
      seg_000002.sqlite
      ...
```

The `.corpus/` directory is the artifact root. Everything inside it is managed by the crate. The corpus root itself is the user's document directory.

### 3.3 Manifest operations

| Method | Description |
|---|---|
| `Manifest::load(corpus_dir)` | Read and parse `manifest.json` |
| `Manifest::save(&self, corpus_dir)` | Atomically write (write to `.tmp`, rename) |
| `Manifest::add_segment(&mut self, meta)` | Append a new segment entry |
| `Manifest::seal_segment(&mut self, id, hash)` | Update entry with hash + sealed_at |
| `Manifest::bump_version(&mut self)` | Increment manifest version |
| `Manifest::segment_path(corpus_dir, id)` | Resolve `segments/seg_{id:06}.sqlite` |
| `Manifest::diff(&self, other) → ManifestDiff` | Compare two manifests (for sync) |

### 3.4 `ManifestDiff` (sync primitive)

```rust
pub struct ManifestDiff {
    /// Segments present in `newer` but not in `older` (by id+hash).
    pub added: Vec<SegmentMeta>,
    /// Segments present in `older` but not in `newer`.
    pub removed: Vec<SegmentId>,
    /// Active segment changed (needs re-upload).
    pub active_changed: bool,
    /// State DB changed.
    pub state_changed: bool,
}
```

This struct is the input to any sync implementation. The corpus crate computes it; a separate sync layer (cloud, rsync, S3, etc.) consumes it.

---

## 4. State DB (`state.rs`)

The state DB is a small **mutable** SQLite file that tracks document-level routing and build metadata. It is the only mutable file that changes on every build (besides the active segment).

### 4.1 Tables

```sql
-- Which document lives in which segment
CREATE TABLE doc_segments (
    doc_id    TEXT PRIMARY KEY,
    path      TEXT NOT NULL,
    file_hash TEXT NOT NULL,      -- blake3 of source file
    segment_id INTEGER NOT NULL,
    indexed_at TEXT NOT NULL       -- RFC 3339
);

-- Tombstones for deleted/changed documents
CREATE TABLE tombstones (
    doc_id     TEXT PRIMARY KEY,
    segment_id INTEGER NOT NULL,
    deleted_at TEXT NOT NULL
);

-- Build metadata
CREATE TABLE build_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
-- Keys: "last_build_at", "schema_version", "corpus_root"
```

### 4.2 Key operations

| Method | Description |
|---|---|
| `State::open(corpus_dir)` | Open or create `state.sqlite` |
| `State::file_index() → HashMap<PathBuf, FileEntry>` | All tracked files with hashes |
| `State::register_doc(doc_id, path, hash, segment_id)` | Record a newly indexed document |
| `State::tombstone_doc(doc_id)` | Mark document as deleted |
| `State::tombstones_in_segment(segment_id) → Vec<DocId>` | For compaction |
| `State::segment_for_doc(doc_id) → Option<SegmentId>` | Routing lookup |
| `State::last_build_at() → Option<DateTime>` | For incremental build decisions |

### 4.3 Sync note

The state DB is small (typically < 1 MB even for large corpora). It can be synced as a single file. Its `build_meta` table includes enough information for a remote to determine if it needs the full state or just a diff.

---

## 5. Build Pipeline (`builder.rs`)

The builder is the main entry point for constructing and updating a corpus.

### 5.1 High-level flow

```
corpus build <dir>
      │
      ▼
┌─────────────────────┐
│  1. Scan directory   │  Recursive walk, filter by supported formats
└─────────┬───────────┘
          ▼
┌─────────────────────┐
│  2. Diff vs state    │  Compare file hashes → new / changed / removed
└─────────┬───────────┘
          ▼
┌─────────────────────┐
│  3. Tombstone        │  Mark removed/changed docs in state DB
└─────────┬───────────┘
          ▼
┌─────────────────────┐
│  4. Ingest new docs  │  Decode → walk → extract chunks → write to segment
└─────────┬───────────┘
          ▼
┌─────────────────────┐
│  5. Rollover check   │  Seal segment if threshold exceeded, open new one
└─────────┬───────────┘
          ▼
┌─────────────────────┐
│  6. Update manifest  │  Write manifest.json
└─────────┘
```

### 5.2 Directory scanning

```rust
pub struct CorpusScanner {
    root: PathBuf,
    /// Glob patterns to exclude (e.g. ".corpus/**", ".git/**", "node_modules/**")
    exclude: Vec<GlobPattern>,
}
```

Uses `walkdir` to traverse. For each file:
1. Check path against exclude patterns.
2. Call `Format::from_path()` to determine format.
3. Check `codecs::supports_from_format(format)` — skip unsupported files.
4. Compute `blake3::hash` of file contents.
5. Yield `ScannedFile { path, format, hash }`.

### 5.3 Change detection

Compare scanned files against `State::file_index()`:

```rust
pub struct BuildPlan {
    /// Files to ingest (new or changed).
    pub to_ingest: Vec<ScannedFile>,
    /// Doc IDs to tombstone (removed or changed — old version).
    pub to_tombstone: Vec<(DocId, SegmentId)>,
    /// Files unchanged (skip).
    pub unchanged: usize,
}
```

Rules:
- **New file** (path not in state): ingest.
- **Changed file** (path in state, different hash): tombstone old doc_id + ingest as new doc.
- **Removed file** (path in state, not on disk): tombstone.
- **Unchanged** (path in state, same hash): skip.

### 5.4 Document ingestion

For each file to ingest:

```rust
async fn ingest_file(
    &self,
    file: &ScannedFile,
    segment: &mut Segment,
    state: &State,
) -> Result<()> {
    // 1. Decode file into Stencila Node
    let (node, _info) = codecs::from_path(&file.path, None).await?;

    // 2. Extract chunks (delegates to schema-specific logic)
    let chunks = extract_chunks(&node, &file.path);

    // 3. Write chunks to active segment
    segment.write_batch(&chunks)?;

    // 4. Register in state
    let doc_id = doc_id_for(&file.path, &file.hash);
    state.register_doc(&doc_id, &file.path, &file.hash, segment.id())?;

    Ok(())
}
```

`extract_chunks()` is the schema-aware piece that will be developed separately. For the initial scaffold, it can produce a single chunk per document with the full text.

### 5.5 Doc ID generation

```rust
/// Deterministic doc ID: blake3(canonical_path).
/// Stable across rebuilds for the same file at the same path.
fn doc_id_for(path: &Path, _hash: &str) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let hash = blake3::hash(canonical.to_string_lossy().as_bytes());
    format!("doc_{}", &hash.to_hex()[..16])
}
```

Doc ID is path-derived (not content-derived) so that a file at the same path gets the same doc ID across content changes — enabling clean tombstone-then-reinsert semantics.

### 5.6 Rollover

After each file ingestion, check:

```rust
if segment.chunk_count() >= self.config.max_chunks_per_segment
    || segment.file_size() >= self.config.max_segment_bytes
{
    let meta = segment.seal()?;
    manifest.seal_segment(meta.id, &meta.hash);
    *segment = Segment::create(&self.segments_dir, manifest.next_segment_id())?;
    manifest.add_segment(SegmentMeta::active(segment.id()));
}
```

### 5.7 Error handling

- Individual file failures are logged and skipped (don't abort the whole build).
- The builder returns a `BuildReport` summarizing successes, failures, and skipped files.
- The active segment and state are always left in a consistent state (SQLite transactions).

### 5.8 Concurrency during build

The build is **sequential per file** for v1 (simplifies state management). Decoding individual files may use internal parallelism (e.g. PDF extraction), but chunk writing is serialized to the active segment.

Future optimization: decode files in parallel with a bounded channel feeding the segment writer.

---

## 6. Parallel Query Engine (`query.rs`)

### 6.1 Design

Queries fan out across all segments in parallel, then merge results.

```
          Query
            │
     ┌──────┼──────┬──────────┐
     ▼      ▼      ▼          ▼
  seg_001 seg_002 seg_003   seg_N    (parallel, bounded)
     │      │      │          │
     ▼      ▼      ▼          ▼
  results results results  results
     │      │      │          │
     └──────┴──────┴──────────┘
            │
         merge (top-k)
            │
         filter tombstones
            │
         final results
```

### 6.2 Core types

```rust
/// Opaque query representation. Schema-specific fields will be added later.
/// For now, this is the minimum needed to drive the parallel execution engine.
pub struct CorpusQuery {
    /// Raw query input (text, filters, etc.) — interpreted by schema layer.
    pub params: QueryParams,
    /// Maximum results to return.
    pub limit: usize,
}

/// A single result row from a segment query.
pub struct QueryHit {
    pub segment_id: SegmentId,
    pub doc_id: String,
    pub chunk_id: String,
    /// Relevance score (higher = better). Schema layer defines scoring.
    pub score: f64,
    /// Opaque payload — schema layer defines what's in here.
    pub payload: serde_json::Value,
}

/// Merged, deduplicated, tombstone-filtered results.
pub struct QueryResult {
    pub hits: Vec<QueryHit>,
    pub segments_searched: usize,
    pub total_hits_before_merge: usize,
}
```

### 6.3 Execution strategy

```rust
pub struct QueryEngine {
    segments_dir: PathBuf,
    manifest: Manifest,
    state: State,
    /// Maximum concurrent segment queries.
    parallelism: usize,  // default: 8
}
```

Key method:

```rust
impl QueryEngine {
    pub async fn search(&self, query: &CorpusQuery) -> Result<QueryResult> {
        // 1. Load tombstone set from state DB
        let tombstones: HashSet<String> = self.state.all_tombstoned_doc_ids()?;

        // 2. Open all segments (sealed RO + active if present)
        let segment_paths = self.manifest.all_segment_paths(&self.segments_dir);

        // 3. Fan out queries with bounded parallelism
        let semaphore = Arc::new(Semaphore::new(self.parallelism));
        let mut handles = Vec::new();

        for path in segment_paths {
            let permit = semaphore.clone().acquire_owned().await?;
            let query = query.clone();
            handles.push(tokio::spawn(async move {
                let segment = Segment::open_readonly(&path)?;
                let hits = segment.execute_query(&query)?;
                drop(permit);
                Ok::<_, eyre::Report>(hits)
            }));
        }

        // 4. Collect results
        let mut all_hits = Vec::new();
        let mut total_before_merge = 0;
        for handle in handles {
            let hits = handle.await??;
            total_before_merge += hits.len();
            all_hits.extend(hits);
        }

        // 5. Filter tombstones
        all_hits.retain(|hit| !tombstones.contains(&hit.doc_id));

        // 6. Sort by score, truncate to limit
        all_hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
        all_hits.truncate(query.limit);

        Ok(QueryResult {
            hits: all_hits,
            segments_searched: segment_paths.len(),
            total_hits_before_merge: total_before_merge,
        })
    }
}
```

### 6.4 Connection pooling

Sealed segments are read-only and can share connections safely. For performance:

- Each `tokio::spawn` task opens its own `rusqlite::Connection` (SQLite connections are not `Send` in the default configuration).
- Alternatively, use `rusqlite::Connection::open_with_flags(SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX)` for maximum read concurrency.
- No connection pool needed — opening a read-only SQLite file is cheap (~microseconds).

### 6.5 Scoring and merge

For v1, scoring is delegated to SQLite FTS5's `rank` function within each segment. Cross-segment merging uses raw score comparison. This is imperfect (BM25 statistics are per-segment, not global), but acceptable for v1 because:

- Most corpora will have 1–3 segments.
- Scores from FTS5 are reasonably comparable across similarly-sized segments.
- A future improvement can compute global IDF from the manifest's chunk counts.

### 6.6 Segment caching (future)

The query engine does not cache segment connections in v1. Future optimization: maintain an LRU cache of open read-only connections keyed by `SegmentId`, evicting when memory pressure is detected.

---

## 7. Cloud Sync Readiness

The corpus crate does **not** implement sync, but its design makes sync straightforward. This section documents the contracts.

### 7.1 Sync-friendly properties

| Property | How the design achieves it |
|---|---|
| **Content-addressable segments** | Each sealed segment has a blake3 hash. Two corpora with the same segment hash have identical data. |
| **Immutable sealed segments** | Sealed segments never change. Sync only needs to upload/download them once. |
| **Small mutable surface** | Only `manifest.json`, `state.sqlite`, and the active segment change between builds. |
| **Manifest diffing** | `Manifest::diff()` produces the exact set of segment uploads/downloads needed. |
| **Deterministic doc IDs** | Path-based doc IDs are stable, so tombstones are meaningful across sync boundaries. |

### 7.2 Sync protocol (for future implementor)

**Push** (local → cloud):

```
1. Compute ManifestDiff(local_manifest, remote_manifest)
2. For each segment in diff.added:
     upload segment file to object storage (key = hash)
3. Upload state.sqlite
4. Upload manifest.json (last — acts as commit)
```

**Pull** (cloud → local):

```
1. Download remote manifest.json
2. Compute ManifestDiff(remote_manifest, local_manifest)
3. For each segment in diff.added:
     download segment file by hash
4. Download state.sqlite
5. Replace local manifest.json
```

**Conflict resolution**: not needed for v1. Corpora are single-writer (one machine builds, others read). Multi-writer support would require segment-level CRDTs or a merge protocol — out of scope.

### 7.3 Object storage layout

Recommended layout for cloud storage (not enforced by the crate):

```
corpus/{corpus_id}/manifest.json
corpus/{corpus_id}/state.sqlite
corpus/{corpus_id}/segments/{hash}.sqlite
```

Using hash as the storage key (not segment ID) enables deduplication and simplifies upload idempotency.

### 7.4 Integration with existing `rust/cloud` crate

The existing cloud crate (`rust/cloud/src/outputs.rs`) already implements ETag-based incremental file upload via:

```rust
POST /v1/workspaces/{id}/site/branches/{branch}/reconcile/{prefix}
```

Segment upload can follow the same pattern:
- Sealed segments: upload once, verify by hash.
- Active segment + state + manifest: upload on every push (small files).

The `ManifestDiff` struct is designed to map directly onto these API calls.

---

## 8. Compaction (future, design only)

Over time, tombstones accumulate in sealed segments. Compaction reclaims space.

```
corpus compact
      │
      ▼
1. Identify segments with high tombstone ratio
   (tombstone_count / chunk_count > threshold, e.g. 30%)
      │
      ▼
2. Read live chunks from those segments
      │
      ▼
3. Write into a new segment
      │
      ▼
4. Seal new segment
      │
      ▼
5. Update manifest (remove old segments, add new)
      │
      ▼
6. Delete old segment files
      │
      ▼
7. Clean tombstones from state DB
```

This is the only operation that "rewrites" segments. It produces new segment IDs and hashes, so sync will treat compacted segments as new uploads.

---

## 9. Public API Surface

### 9.1 Library API (`lib.rs`)

```rust
pub struct Corpus {
    root: PathBuf,
    manifest: Manifest,
    state: State,
}

impl Corpus {
    /// Open an existing corpus or initialize a new one.
    pub fn open(root: impl AsRef<Path>) -> Result<Self>;

    /// Build/update the corpus index from the document directory.
    pub async fn build(&mut self, config: BuildConfig) -> Result<BuildReport>;

    /// Execute a query across all segments.
    pub async fn query(&self, query: CorpusQuery) -> Result<QueryResult>;

    /// Seal the active segment (e.g. before sync).
    pub fn seal(&mut self) -> Result<()>;

    /// Compute the diff needed to sync to/from a remote manifest.
    pub fn diff(&self, remote: &Manifest) -> ManifestDiff;

    /// List all segments with metadata.
    pub fn segments(&self) -> &[SegmentMeta];

    /// Corpus statistics.
    pub fn stats(&self) -> CorpusStats;
}

pub struct BuildConfig {
    pub max_chunks_per_segment: u64,
    pub max_segment_bytes: u64,
    pub exclude_patterns: Vec<String>,
    /// If true, skip files whose hash hasn't changed (default: true).
    pub incremental: bool,
}

pub struct BuildReport {
    pub files_scanned: usize,
    pub files_ingested: usize,
    pub files_unchanged: usize,
    pub files_tombstoned: usize,
    pub files_failed: Vec<(PathBuf, String)>,
    pub chunks_written: usize,
    pub segments_sealed: usize,
    pub duration: Duration,
}
```

### 9.2 CLI integration

The CLI will add subcommands (likely under `stencila corpus`):

```
stencila corpus build [<dir>]         # build/update index
stencila corpus query <text> [flags]  # search
stencila corpus status                # show corpus stats
stencila corpus seal                  # seal active segment
```

These map directly to `Corpus::build()`, `Corpus::query()`, etc.

---

## 10. Implementation Phases

### Phase 1: Skeleton + segment lifecycle (est. ~2 days)

- [ ] Create `rust/corpus` crate scaffold
- [ ] Implement `Segment::create()`, `write_batch()`, `seal()`, `open_readonly()`
- [ ] Implement `Manifest` read/write/diff
- [ ] Implement `State` DB open/read/write
- [ ] Placeholder `schema.rs` with a trivial table (id, text)
- [ ] Unit tests: create segment, write rows, seal, verify hash, open RO

### Phase 2: Build pipeline (~3 days)

- [ ] Implement `CorpusScanner` (directory walk + format detection + hashing)
- [ ] Implement change detection (diff scanned files vs state)
- [ ] Implement `Builder` orchestrator (tombstone → ingest → rollover → manifest)
- [ ] Wire up `codecs::from_path()` for document decoding
- [ ] Stub `extract_chunks()` (one chunk per doc, full text)
- [ ] Integration test: build corpus from test fixtures, verify segment contents

### Phase 3: Parallel query engine (~2 days)

- [ ] Implement `QueryEngine::search()` with bounded parallelism
- [ ] Implement tombstone filtering
- [ ] Implement score-based merge + top-k truncation
- [ ] Integration test: build corpus, query across multiple segments, verify results

### Phase 4: Public API + CLI (~1 day)

- [ ] Implement `Corpus` facade (open/build/query/seal/diff/stats)
- [ ] Add CLI subcommands (`corpus build`, `corpus query`, `corpus status`)
- [ ] End-to-end test: CLI build + query on a sample directory

### Phase 5: Schema iteration (separate track)

- [ ] Design chunk schema (fields, FTS config, citation tables)
- [ ] Implement `extract_chunks()` with structure-aware walking
- [ ] Add node_type filtering, citation constraints to query
- [ ] Migrate `init_schema()` from placeholder to real DDL

---

## 11. Key Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| **SQLite over Kuzu** | SQLite (rusqlite) | Single-file segments, content-addressable, no daemon, FTS5 built-in, universal tooling |
| **blake3 over MD5/SHA** | blake3 | Fast (SIMD), streaming, used in modern content-addressed storage; replaces MD5 ETags from cloud crate |
| **Path-based doc IDs** | `blake3(canonical_path)` | Stable across content changes, enables clean tombstone semantics |
| **Sequential file ingestion** | Single-threaded writer | Simplifies state management; parallelism applied at query time instead |
| **Bounded query parallelism** | 8 default tasks | Balances throughput vs resource usage; SQLite read-only is thread-safe |
| **Manifest as commit point** | Write manifest last | Crash before manifest write = no visible change (atomic semantics) |
| **Schema in code, not embedded files** | `schema.rs` | Schema changes are compile-time checked; no runtime migration complexity for v1 |

---

## 12. Risks & Mitigations

| Risk | Mitigation |
|---|---|
| FTS5 BM25 scores not comparable across segments | Acceptable for v1 (few segments); future: compute global IDF from manifest chunk counts |
| Large PDFs block the build pipeline | Per-file timeout + error isolation (log and skip) |
| SQLite WAL files leak into segment directory | Sealed segments use `journal_mode=OFF`; active segment uses `journal_mode=WAL` but is excluded from sync |
| Schema changes require full rebuild | Acceptable for v1; future: versioned migration system like node-db |
| Active segment corruption on crash | WAL mode provides crash safety; worst case = re-ingest files added since last seal |
