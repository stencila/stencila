---
title: "File Digest Record"
description: "Digest of a file relevant to reproduction."
---

# File Digest Record

Digest of a file relevant to reproduction.

This record is used for lockfiles and other local files where disclosing the
path and digest is useful but embedding the full file would be too large or
private.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`path`](#path) | `string` | No | File path. |
| [`digest`](#digest) | `string` | No | Digest of the file contents. |

### `path`

File path.

Paths are optional because they can reveal private project structure.
When omitted, the digest can still identify the file content.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `digest`

Digest of the file contents.

The value should use `algorithm:hex` form. The legacy `sha256` field name
is accepted for unpublished pre-v1 payloads.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
