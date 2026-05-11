---
title: "Source Record"
description: "Source-control facts for the signed output."
---

# Source Record

Source-control facts for the signed output.

Source information helps reviewers and automated systems locate the authored
document state that led to the signed asset. Each field is optional because
privacy policy or offline workflows may disclose only a subset.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`repository`](#repository) | `string` | No | Source repository URL or identifier. |
| [`commit`](#commit) | `string` | No | Source commit hash or revision identifier. |
| [`path`](#path) | `string` | No | Path to the source document within the repository or project. |
| [`dirty`](#dirty) | `boolean` | No | Whether uncommitted changes were present. |
| [`patchDigest`](#patch-digest) | `string` | No | Digest of an unpublished patch, when disclosed. |
| [`tag`](#tag) | `string` | No | Source tag or release identifier. |

### `repository`

Source repository URL or identifier.

This may be a public URL, an internal repository identifier, or a redacted
stable label. It is optional because repository names can be sensitive.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `commit`

Source commit hash or revision identifier.

The field name is intentionally generic enough for Git and non-Git
systems, while the documentation calls out the common commit use case.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `path`

Path to the source document within the repository or project.

Paths are useful for reproducibility but can leak project structure, so
they are optional and may be redacted under the privacy policy.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `dirty`

Whether uncommitted changes were present.

This boolean is optional rather than defaulting to false because "not
assessed" and "clean" are different provenance states.

**Type:** `boolean` | **Required:** No | **Nullable:** Yes

### `patchDigest`

Digest of an unpublished patch, when disclosed.

The value should use `algorithm:hex` form. A patch digest lets a signer
attest that local changes existed without embedding the patch itself.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `tag`

Source tag or release identifier.

Tags help link the signed asset to a release even when a full commit hash
is not appropriate for disclosure.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
