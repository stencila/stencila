---
title: Under the Hood
description: How workspace database sync works — SQLite sessions, content addressing, append-only tables, and schema migrations
---

This page explains the internals of the workspace database sync system. You don't need to understand these details to use `stencila db push` and `stencila db pull`, but they help if you're troubleshooting, contributing to Stencila, or just curious about the design.

# SQLite Session extension

Stencila uses SQLite's [Session extension](https://www.sqlite.org/sessionintro.html) to compute the differences between two database states. The Session extension can produce a compact binary **changeset** that describes every INSERT, UPDATE, and DELETE needed to transform one database into another.

Rather than keeping a persistent session open for the lifetime of a database connection, Stencila computes changesets on-demand at push time using `Session::diff`. This compares your live database against a reconstructed copy of the manifest head (the base snapshot with all existing changesets replayed in order). The approach has several advantages:

- **No lifecycle complexity.** The database open path doesn't need to manage session attachment or detachment.
- **Cross-process safety.** Changes made by any process — including external SQLite tools — are captured, since the diff compares the full current state.
- **Crash resilience.** No risk of losing accumulated session state if a process crashes before pushing.
- **Correctness.** The diff always produces a minimal, correct changeset regardless of how changes were made.

The tradeoff is that push must reconstruct the manifest head to compute the diff. This is mitigated by the local blob cache (downloaded blobs are reused) and by automatic snapshot rotation (which resets the changeset list, keeping reconstruction fast).

# Append-only tables and UUIDs

The database schema is designed to minimize conflicts when multiple contributors push changes. The key design choices are:

**UUID primary keys.** Tables like `workflow_runs`, `workflow_interviews`, and `workflow_artifacts` use [UUIDv7](https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7) primary keys (e.g. `run_id TEXT PRIMARY KEY`). UUIDv7 is time-ordered and globally unique, so two contributors generating rows independently will never collide on primary keys.

**Append-only access patterns.** Most tables are effectively append-only — new workflow runs insert new rows rather than updating existing ones. When updates do occur (e.g. setting `completed_at` on a run), they target rows that belong to the current contributor's run, avoiding cross-contributor conflicts.

**Scoped foreign keys.** Child tables (nodes, edges, context, outputs) are keyed by `(run_id, ...)`, so data from different runs never overlaps.

These patterns mean that changesets from different contributors typically touch disjoint rows, making conflict-free merging the common case. The changeset conflict handler uses `REPLACE` for data conflicts (remote wins), `OMIT` for not-found rows (the intent — deletion — is already satisfied), and `ABORT` for constraint violations (which indicate a bug).

# Content addressing

All blobs (snapshots and changesets) are identified by their SHA-256 hash. The manifest references hashes, not file paths or sequence numbers. When downloading a blob, the client recomputes the hash and verifies it matches. This provides:

- **Deduplication.** Identical content uploaded by different contributors is stored once.
- **Tamper detection.** A corrupted or modified blob is detected before it's applied.
- **Integrity verification.** The local blob cache verifies hashes on read, discarding any blob that fails the check.

# Manifest format

The manifest at `.stencila/db.json` is a JSON file tracked by git:

```json
{
  "format": "stencila-db-sync-v1",
  "schema_version": { "workflows": 1 },
  "base_snapshot": {
    "hash": "a1b2c3d4e5f6...",
    "compression": "zstd",
    "schema_version": { "workflows": 1 },
    "created_at": "2026-02-15T10:00:00Z",
    "size": 3200000,
    "message": "compact after batch processing"
  },
  "changesets": [
    {
      "hash": "d4e5f6a7b8c9...",
      "schema_version": { "workflows": 1 },
      "created_at": "2026-02-18T09:15:00Z",
      "size": 48200,
      "message": "batch-1 results"
    }
  ]
}
```

Key fields:

- **`format`**: manifest version identifier, currently `stencila-db-sync-v1`.
- **`schema_version`**: the maximum applied migration version per domain. Used to detect schema changes.
- **`base_snapshot`**: the full database snapshot that forms the baseline. Compression is `zstd`.
- **`changesets`**: an ordered list of incremental changesets to apply after the snapshot.
- **`message`**: optional human-readable description provided via `-m` on push or snapshot.
- **`size`**: byte size of each blob as stored (after compression). Used for progress bars and compaction heuristics.

# Snapshot compression

Snapshots are compressed with [Zstandard](https://facebook.github.io/zstd/) (zstd) at level 3 before upload. Changesets are not compressed — they are already compact binary diffs produced by the Session extension. The `compression` field in the manifest allows future format changes without breaking existing manifests.

# Schema migrations

Each domain (e.g. `workflows`) has a set of numbered migrations. Migrations are expected to be **additive only** — they should add columns, tables, or indexes, and avoid dropping/renaming/changing column types. The sync design relies on this so older changesets can still be applied to newer schemas.

## Automatic snapshot rotation

The manifest records the schema version at the time each changeset was created. On push, if the local schema version differs from the manifest's `schema_version`, Stencila automatically creates a full snapshot instead of a changeset:

```
⚡ Schema changed (workflows@1 → workflows@2), creating snapshot instead of changeset
```

The manifest resets: new base snapshot, changeset list cleared, schema version updated. This happens transparently — no user action is needed. The only visible effect is that the first push after a migration is larger (a full snapshot rather than an incremental changeset).

## Why this works

Because the manifest is git-tracked, `git pull` delivers both the new migration code and the updated manifest at the same time. When the database is opened, `migrate()` runs before any changesets are applied. So by the time changesets are replayed, the consumer's schema matches the producer's.

For the edge case where a developer updates their code (pulling a new migration) before pulling new database state:

1. Their database is auto-migrated to the new schema version on open.
2. Existing changesets from before the migration are applied successfully, because migrations are additive-only.
3. On their next push, the schema mismatch triggers a snapshot, cleanly resetting the epoch.

# Automatic compaction

To prevent changeset lists from growing without bound, Stencila automatically rotates to a new snapshot when:

- The changeset count reaches **50**, or
- The existing cumulative changeset size is already at least **50 MB**

This keeps reconstruction time bounded during push and pull.

# Atomicity and safety

All operations that modify the local database (pull, reset) build state into a **temporary file** and then atomically rename it to replace the working database. If a crash or network error occurs mid-operation, the working database is untouched.

Blobs are always uploaded **before** the manifest is written. If a push succeeds in uploading blobs but fails to write the manifest, no data is lost — the blobs are inert until referenced by a committed manifest. Retrying the push deduplicates by hash.
