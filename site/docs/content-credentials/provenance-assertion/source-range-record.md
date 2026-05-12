---
title: "Source Range Record"
description: "Range coordinates in the source document."
---

# Source Range Record

Range coordinates in the source document.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`startLine`](#start-line) | `integer` | Yes | 1-based index of the first line in the source range. |
| [`startColumn`](#start-column) | `integer` | Yes | 1-based UTF-8 column index of the start of the source range. |
| [`endLine`](#end-line) | `integer` | Yes | 1-based index of the line containing the exclusive end position. |
| [`endColumn`](#end-column) | `integer` | Yes | 1-based UTF-8 column index of the exclusive end position. |

### `startLine`

1-based index of the first line in the source range.

**Type:** `integer` | **Required:** Yes

### `startColumn`

1-based UTF-8 column index of the start of the source range.

**Type:** `integer` | **Required:** Yes

### `endLine`

1-based index of the line containing the exclusive end position.

**Type:** `integer` | **Required:** Yes

### `endColumn`

1-based UTF-8 column index of the exclusive end position.

**Type:** `integer` | **Required:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
