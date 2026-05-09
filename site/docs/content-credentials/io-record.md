---
title: "IO Record"
description: "Input or output entity used by a provenance activity."
---

# IO Record

Input or output entity used by a provenance activity.

The same shape is used for inputs and outputs so workflows can describe
files, datasets, prompts, parameters, document nodes, and generated artifacts
using one compact record.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`id`](#id) | `string` | No | Optional input or output identifier. |
| [`kind`](#kind) | `string` | No | Input or output kind. |
| [`name`](#name) | `string` | No | Input or output name. |
| [`uri`](#uri) | `string` | No | Input or output URI. |
| [`mediaType`](#media-type) | `string` | No | IANA media type for the input or output. |
| [`digest`](#digest) | `string` | No | Digest of the input or output bytes or value. |
| [`version`](#version) | `string` | No | Input or output version. |
| [`access`](#access) | `string` | No | Access level or policy applied to the input or output. |
| [`redaction`](#redaction) | `string` | No | Redaction applied to this input or output. |
| [`size`](#size) | `integer` | No | Byte length. |
| [`width`](#width) | `integer` | No | Width for image, video, or tabular outputs. |
| [`height`](#height) | `integer` | No | Height for image or video outputs. |
| [`rowCount`](#row-count) | `integer` | No | Row count for tabular outputs. |
| [`columnCount`](#column-count) | `integer` | No | Column count for tabular outputs. |
| [`metadata`](#metadata) | any JSON value | No | Structured IO metadata chosen by the producer. |

### `id`

Optional input or output identifier.

IDs allow `activity.usedInputIds` and `activity.generatedOutputIds` to
express relationships without duplicating entity metadata.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `kind`

Input or output kind.

Suggested values include `file`, `dataset`, `parameter`, `prompt`,
`document-node`, `artifact`, and `asset`; extension values should use a
reverse-DNS prefix when outside Stencila's vocabulary.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `name`

Input or output name.

Names are human-facing and optional because they can be private or absent
for generated intermediate values.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `uri`

Input or output URI.

URIs can locate external data or artifacts. They are optional and should
be redacted when they contain secrets, private hostnames, or signed URLs.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `mediaType`

IANA media type for the input or output.

Media type is optional because parameters and abstract document nodes may
not have byte-oriented media types.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `digest`

Digest of the input or output bytes or value.

The value should use `algorithm:hex` form. It is the preferred identity
for disclosed files and values because names and URIs can change.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `version`

Input or output version.

Version is useful for datasets, packages, models, and artifacts that have
a logical release identity in addition to a digest.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `access`

Access level or policy applied to the input or output.

Suggested values include `public`, `private`, `restricted`, `redacted`,
and `undisclosed`. This explains why a URI or digest may be absent.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `redaction`

Redaction applied to this input or output.

This is a compact per-entity redaction signal. Detailed policy and field
paths belong in `privacy.redactions`.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `size`

Byte length.

Size is optional because not all IO entities are byte streams and because
producers may choose not to disclose exact sizes.

**Type:** `integer` | **Required:** No | **Nullable:** Yes

### `width`

Width for image, video, or tabular outputs.

Width is optional and interpreted in the natural unit for the media type,
usually pixels for images and columns for some tabular projections.

**Type:** `integer` | **Required:** No | **Nullable:** Yes

### `height`

Height for image or video outputs.

Height is optional and usually measured in pixels.

**Type:** `integer` | **Required:** No | **Nullable:** Yes

### `rowCount`

Row count for tabular outputs.

Row count is useful for data outputs and can be disclosed even when the
table contents are private.

**Type:** `integer` | **Required:** No | **Nullable:** Yes

### `columnCount`

Column count for tabular outputs.

Column count complements `rowCount` for quick dataset inspection.

**Type:** `integer` | **Required:** No | **Nullable:** Yes

### `metadata`

Structured IO metadata chosen by the producer.

This field is for domain-specific metadata that is intentionally carried
as JSON rather than promoted into the stable schema. The legacy `extra`
field name is accepted for unpublished payloads.

**Type:** any JSON value | **Required:** No


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
