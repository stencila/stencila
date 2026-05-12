---
title: "Node Record"
description: "Facts about a Stencila node related to the signed asset."
---

# Node Record

Facts about a Stencila node related to the signed asset.

This record anchors C2PA bytes back to Stencila document structure while
staying compact enough for manifests. It stores stable node identity and
selected public metadata rather than embedding private source content.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`nodeType`](#node-type) | `string` | Yes | Stencila Schema node type. |
| [`nodeId`](#node-id) | `string` | No | Stable Stencila node identifier, when available. |
| [`persistentId`](#persistent-id) | `string` | No | Author-supplied persistent identifier from the Stencila Schema `id` field. |
| [`nodePath`](#node-path) | `string` | No | Path to the node within the document tree. |
| [`sourceRange`](#source-range) | [`SourceRangeRecord`](source-range-record) | No | Range of the node in the source document. |
| [`labelType`](#label-type) | `string` | No | Stencila label type for labelled nodes. |
| [`label`](#label) | `string` | No | Stencila label for the node. |
| [`title`](#title) | `string` | No | Human-readable node or work title. |
| [`programmingLanguage`](#programming-language) | `string` | No | Programming language for executable nodes. |
| [`contentUrl`](#content-url) | `string` | No | URL or path for media-like nodes. |
| [`mediaType`](#media-type) | `string` | No | IANA media type for media-like nodes. |

### `nodeType`

Stencila Schema node type.

Values such as `CodeChunk`, `Figure`, `Table`, `Article`, or `File` let
Stencila-aware consumers recover the kind of work represented by the
asset without depending on media type heuristics. Values intentionally
use Stencila Schema's `PascalCase` node type convention.

**Type:** `string` | **Required:** Yes

### `nodeId`

Stable Stencila node identifier, when available.

This is a deterministic structural identifier derived from the node's
content, label, or position in the stabilized document tree (e.g.
`content-1-content-3`). It is stable across re-renders of the same
source document and lets later tooling correlate credentials with the
source document tree. It is optional because standalone asset signing
and imported files may not have a stable Stencila node ID.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `persistentId`

Author-supplied persistent identifier from the Stencila Schema `id`
field.

This is the DOM-style identifier the document author wrote, separate
from the structural `nodeId`. When the author has set an `id` (for
example `id: "fig-plot"` on a Figure), it is recorded here unchanged so
verifiers can locate the node by its author-given name without needing
to know Stencila's structural identifier scheme.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `nodePath`

Path to the node within the document tree.

A path is useful when IDs are absent or when reviewers need to locate the
represented node in a specific document snapshot. It is optional because
paths can reveal document structure and can be unstable across edits.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `sourceRange`

Range of the node in the source document.

Positions are 1-based UTF-8 line and column coordinates, with an
exclusive end position. The range covers the whole serialized node in
the source document, not only one of its properties.

**Type:** [`SourceRangeRecord`](source-range-record) | **Required:** No | **Nullable:** Yes

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

### `contentUrl`

URL or path for media-like nodes.

This is useful for output nodes such as `ImageObject`, `AudioObject`,
`MediaObject`, and `VideoObject`, but is optional because many Stencila
nodes are not byte-backed media references.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `mediaType`

IANA media type for media-like nodes.

This is node metadata. The signed bytes remain described by
`asset.mediaType`, which may differ for alternate renditions.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
