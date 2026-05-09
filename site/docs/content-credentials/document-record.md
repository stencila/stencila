---
title: "Document Record"
description: "Facts about the Stencila node or work represented by the signed asset."
---

# Document Record

Facts about the Stencila node or work represented by the signed asset.

This record anchors C2PA bytes back to Stencila document structure while
staying compact enough for manifests. It stores stable node identity and
execution-affecting digests rather than embedding private source content.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`nodeType`](#node-type) | `string` | Yes | Stencila Schema node type. |
| [`nodeId`](#node-id) | `string` | No | Stable Stencila node identifier, when available. |
| [`nodePath`](#node-path) | `string` | No | Path to the node within the document tree. |
| [`labelType`](#label-type) | `string` | No | Stencila label type for labelled nodes. |
| [`label`](#label) | `string` | No | Stencila label for the node. |
| [`title`](#title) | `string` | No | Human-readable node or work title. |
| [`programmingLanguage`](#programming-language) | `string` | No | Programming language for executable nodes. |
| [`executionDigest`](#execution-digest) | [`ExecutionDigestRecord`](execution-digest-record) | No | Digests representing executable node state at signing time. |

### `nodeType`

Stencila Schema node type.

Values such as `CodeChunk`, `Figure`, `Table`, `Article`, or `File` let
Stencila-aware consumers recover the kind of work represented by the
asset without depending on media type heuristics.

**Type:** `string` | **Required:** Yes

### `nodeId`

Stable Stencila node identifier, when available.

Node IDs let later tooling correlate credentials with the source document
tree. They are optional because standalone asset signing and imported
files may not have a stable Stencila node ID.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `nodePath`

Path to the node within the document tree.

A path is useful when IDs are absent or when reviewers need to locate the
represented node in a specific document snapshot. It is optional because
paths can reveal document structure and can be unstable across edits.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `labelType`

Stencila label type for labelled nodes.

Label type distinguishes figure, table, equation, and other label
namespaces without parsing the label string itself.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `label`

Stencila label for the node.

This repeats the asset label when the signed asset is a direct rendition
of a labelled node. Keeping it here allows document-level verification
even when the asset record is used for an unlabelled file rendition.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `title`

Human-readable node or work title.

Titles improve audit readability but remain optional because many
executable nodes and generated outputs are intentionally untitled.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `programmingLanguage`

Programming language for executable nodes.

Language belongs on the document node, not only the kernel, because it is
part of the authored source semantics that affect execution and review.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `executionDigest`

Digests representing executable node state at signing time.

These compactly attest the state that Stencila considered relevant to
generated output, without disclosing the source code or dependency values
themselves.

**Type:** [`ExecutionDigestRecord`](execution-digest-record) | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
