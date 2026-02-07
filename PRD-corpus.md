# PRD: Stencila Rust CLI — rust/corpus Crate (Segmented Search DB Builder)

## Owner

Stencila Rust / CLI team

## Status

Draft

## Summary

Build a new Rust crate, `rust/corpus`, that constructs and queries a local segmented SQLite search DB from a corpus of documents (any format supported by Stencila), including files in subdirectories.

This crate will supersede `rust/node-db` while retaining (and expanding) its core differentiator: structure-aware indexing and search over Stencila documents.

The output is a set of immutable SQLite segment files plus a small manifest/state, designed for efficient sync and portability.

---

## Background / Motivation

Stencila currently has `rust/node-db`, which supports structured indexing/search over Stencila document nodes.

New requirements have emerged:

1. **Append-mostly scaling**: corpora grow over time.
2. **Syncability**: users and agents need to push/pull the DB without rebuilding.
3. **Portability**: DB artifacts should work well with object storage and Stencila Cloud v1 sync APIs.
4. **Multi-surface usage**: the same DB substrate supports:
   - Stencila Library (user documents)
   - Agent memory
   - Pod shared memory

A monolithic SQLite file is not suitable because it syncs poorly (small changes rewrite large files). The architecture now uses immutable SQLite segments.

---

## Goals

### Functional

- Build a segmented SQLite search DB from a corpus directory:
  - supports Markdown, PDFs, notebooks, and any Stencila-supported format
  - recursively includes subdirectories
- Perform structure-aware search:
  - node type filtering (Paragraph, Sentence, Citation, Figure, Table, Image, etc.)
  - fielded search (text, captions, alt text, titles, authors, references)
  - relational search (e.g. "sentences mentioning X that cite Y and Z")
- Provide a query API usable by:
  - the Stencila CLI
  - agent/pod subsystems

### Architecture

- Replace `node-db` with a segmented design:
  - immutable segment SQLite DBs
  - manifest + small state DB (tombstones/routing)
- Keep build incremental and append-friendly.

### Integration

- Produce artifacts compatible with Cloud sync APIs (v1) but do not implement sync yet (will be layered later).

---

## Non-goals (v1)

- Sync implementation (push/pull) — Cloud API exists; will be integrated after.
- Hosted/server-side indexing or search.
- Perfect "SQL query language" for end-users.
- Full document extraction pipeline for every format (PDF extraction quality can improve later).

---

## Target Users / Use Cases

### 1) Stencila Library (user-facing)

User has:

```
library/
  papers/
  notes/
  pdfs/
```

They want:
- build DB once
- query locally
- later sync it

### 2) Agent memory (system-facing)

Agent writes:

```
agents/alice/memory/
  2026-02-07.md
  decisions.md
  scratchpad.md
```

They want:
- fast incremental build
- structured retrieval for continuity

---

## Key Product Requirements

### PR1 — Corpus ingestion

- Input: a root directory.
- Traverse recursively.
- For each file:
  - determine if Stencila can read it
  - decode into Stencila document model (or partial model)
  - extract structured nodes
  - chunk into indexable units

### PR2 — Structure-aware indexing

Indexing must preserve document structure.

Example query types:
- "All sentences that mention protein folding"
- "All paragraphs that cite doi:10.1038/..."
- "Sentences mentioning X that cite Y and Z"
- "All figures whose caption contains workflow"
- "All tables that mention p-value"

This implies indexing must include:
- node type
- node path / ancestry
- citations within a text span
- reference identifiers (DOI, URL, citation keys)
- local file provenance

### PR3 — Segmented SQLite output

Output layout (conceptual; final naming TBD):

```
<dataset-root>/
  db/
    segments/
      seg_000001.sqlite
      seg_000002.sqlite
    manifest.json
    state.sqlite
```

Segments are immutable once sealed.

### PR4 — Incremental build (append-mostly)

- Detect new/changed/removed files.
- Default behavior:
  - new files → append to current segment
  - removed files → tombstone
  - changed files → treat as remove+add (append-only semantics)

### PR5 — Query API

Provide a Rust API for:
- text search (FTS)
- vector search (optional in v1, but schema must support it)
- structure filters (node types, citation constraints)

The CLI will wrap this later.

---

## Proposed Crate Organization

### `rust/corpus` responsibilities

- corpus traversal
- file ingestion + conversion into Stencila nodes
- chunk extraction
- segment writer (SQLite)
- manifest/state management
- query execution across segments

### What moves out of / replaces `rust/node-db`

- `node-db`'s schema and indexing logic should be ported and extended.
- `node-db`'s concept of "node records" remains, but stored in segments.

---

## Data Model (Index Records)

### Core record: Chunk

A chunk is the unit of retrieval.

Minimum fields:
- `chunk_id` (stable)
- `doc_id`
- `path`
- `node_type` (e.g. Sentence, Paragraph, FigureCaption)
- `node_id` (Stencila node id)
- `parent_node_id` (optional)
- `text` (retrieval text)
- `citations` (normalized ids: DOI, URL, citekey)
- `metadata` (JSON: headings, figure number, etc.)
- `embedding` (optional)

### Why sentence-level indexing matters

It enables:
- "all sentences mentioning X"
- citation co-occurrence constraints

Paragraph-level chunks can be stored too, but sentence-level is needed for the examples.

---

## SQLite Segment Schema (v1)

Each `seg_*.sqlite` should contain:
- `chunks` table (row per chunk)
- FTS virtual table over `chunks.text`
- citation mapping table(s)

Example tables (conceptual):

- `chunks(chunk_id PRIMARY KEY, doc_id, path, node_type, node_id, parent_node_id, text, metadata_json)`
- `chunk_citations(chunk_id, citation_id)` where `citation_id` is normalized
- `citations(citation_id PRIMARY KEY, kind, value)`
  (`kind` = doi|url|citekey|pmid|arxiv etc.)
- `docs(doc_id PRIMARY KEY, path, hash, created_at)` (optional)

FTS:

- `chunks_fts(text, chunk_id UNINDEXED, node_type UNINDEXED, doc_id UNINDEXED)`

Vector:

- optional, but reserve a table shape so it can be added without migration pain.

---

## Manifest + State (non-segment)

### `manifest.json`

Tracks:
- schema version
- segment list (segment id, hash, size, sealed_at)
- embedding model id (if embeddings enabled)
- chunking strategy version

### `state.sqlite`

Mutable small DB for:
- `doc_id` -> `segment_id`
- tombstones (deleted doc ids)
- last build timestamp
- optional: file hash cache

---

## Query Semantics (v1)

### Required query operations

- Full text search with ranking.
- Filter by:
  - `node_type`
  - `path` prefix
- Citation constraints:
  - include citation(s): chunk must cite Y
  - include-all: chunk must cite Y and Z
  - exclude citation(s)

### Example query API

Not user-facing syntax, but Rust-level:

```rust
Query {
  text: Some("protein folding"),
  node_types: vec![NodeType::Sentence],
  must_cite: vec!["doi:10.1038/...".into()],
  must_cite_all: vec!["doi:...".into(), "doi:...".into()],
  path_prefix: Some("papers/".into()),
  limit: 50
}
```

---

## Segment Strategy

### Default segment sizing

- rollover at:
  - ~250–1000MB OR
  - ~100k chunks
- segments are sealed and immutable.

### Parallel query

Query all segments in bounded parallelism (8–16 tasks) and merge top-k.

---

## Build Strategy

### File change detection

Maintain a file index:
- path
- file hash (blake3)
- last indexed timestamp
- doc_id

Rules:
- new file → ingest
- changed hash → tombstone old doc_id, ingest new
- missing file → tombstone doc_id

### Append-only semantics

No rewriting old segments during normal builds.

### Compaction (later)

Optional future command:
- merge segments
- drop tombstoned docs
- rebuild FTS

---

## Integration Points

### Stencila document conversion

`rust/corpus` must rely on existing Stencila parsing/decoding pipelines (wherever they currently live).

It must produce Stencila node structures sufficient for:
- node type
- text extraction
- citations extraction
- figures/tables captions, etc.

### Embeddings (v1 optional)

The crate should be designed so embeddings can be enabled later, but not required for initial delivery.

---

## Deliverables

### D1 — New crate scaffold

- `rust/corpus` created
- build + test in CI
- minimal docs

### D2 — Segment writer

- create segment SQLite
- write chunks + FTS index
- seal segments

### D3 — Corpus ingestion

- traverse directory
- ingest Markdown + Stencila-native formats first
- stub for PDFs/notebooks with fallback text extraction

### D4 — Query engine

- FTS queries across segments
- `node_type` + citation constraints

### D5 — Compatibility layer / migration plan

- parity checklist vs `node-db`
- mapping of old `node-db` APIs to new ones

---

## Acceptance Criteria

### Build

- Given a corpus directory with subfolders, build produces:
  - ≥1 `seg_*.sqlite`
  - `manifest.json`
  - `state.sqlite`
- Adding a new document results in:
  - no changes to old segments
  - a change only to active segment + state/manifest

### Query

- Queries can:
  - search sentences mentioning a term
  - filter to node types
  - enforce citation constraints

### Performance

- Works interactively for corpora up to:
  - ~10k documents
  - ~1M chunks
  - (Targets can be refined later.)

---

## Risks / Challenges

- Sentence extraction quality across formats.
- Citation extraction and normalization across formats.
- SQLite extension choice for vectors (if/when enabled).
- Query merging across segments (ranking consistency).

---

## Proposed Implementation Plan (Engineering)

### Phase 0 — Port node-db concepts

- Identify `node-db` schema and indexing logic to reuse.
- Define stable Chunk model.

### Phase 1 — Minimal segmented FTS DB

- Segment writer
- FTS search
- `node_type` filters

### Phase 2 — Citation-aware indexing

- citation normalization
- `chunk_citations` table
- query constraints

### Phase 3 — Incremental builds

- state DB for file hashes
- append-only rebuild logic
- tombstones

### Phase 4 — Format expansion

- PDFs, notebooks
- better extraction pipelines

### Phase 5 — Embeddings (optional)

- add embedding column/table
- add vector extension + similarity search
- hybrid scoring

---

## Notes for the team

- The Cloud team has finished implementing v1 sync APIs; details will be provided separately.
- The CLI work should focus on producing correct local DB artifacts and a clean crate API.
- Sync integration can be added after `rust/corpus` stabilizes.
