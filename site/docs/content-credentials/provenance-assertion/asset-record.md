---
title: "Asset Record"
description: "Facts about the source asset entity."
---

# Asset Record

Facts about the source asset entity.

The asset record is deliberately byte-oriented, but it is not a duplicate of
C2PA's own hard-binding assertions. It stores media type, size, dimensions,
and a digest of the pre-credential content so Stencila-aware consumers can
identify the source rendition without embedding a full Stencila node.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`id`](#id) | `string` | No | Optional asset identifier used by activity references. |
| [`assetType`](#asset-type) | `string` | Yes | Stencila or C2PA-facing asset type. |
| [`role`](#role) | `string` | No | Asset role in the Stencila export context. |
| [`mediaType`](#media-type) | `string` | Yes | IANA media type for the asset bytes. |
| [`contentDigest`](#content-digest) | `string` | Yes | Digest of the pre-credential asset bytes. |
| [`label`](#label) | `string` | No | Stencila label associated with the asset. |
| [`title`](#title) | `string` | No | Human-readable title for the asset. |
| [`size`](#size) | `integer` | No | Asset byte length before signing. |
| [`width`](#width) | `integer` | No | Width for image or video assets. |
| [`height`](#height) | `integer` | No | Height for image or video assets. |

### `id`

Optional asset identifier used by activity references.

The digest is the cryptographic identity, but a short ID is useful when
`activities.generatedAssetIds` or external reports need to refer to this
asset without repeating a long digest.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `assetType`

Stencila or C2PA-facing asset type.

The initial vocabulary is `asset`, `image`, `figure`, `table`, `dataset`,
and `document`; reverse-DNS extension values are allowed. The value is a
broad class for UI and policy decisions, not a replacement for
`mediaType`.

**Type:** `string` | **Required:** Yes

### `role`

Asset role in the Stencila export context.

The role captures why this asset exists, for example `document-export`,
`figure`, `table-image`, or `computational-output`. It complements
`assetType`, which remains the broad media or entity class.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `mediaType`

IANA media type for the asset bytes.

Media type is required because C2PA validators and reproducibility tools
need to know how the bytes should be interpreted independently of file
extension or URL.

**Type:** `string` | **Required:** Yes

### `contentDigest`

Digest of the pre-credential asset bytes.

The value should use `algorithm:hex` form, for example `sha256:...`.
Keeping the algorithm in the value avoids baking `sha256` into every
field name and leaves room for future digest algorithms. This digest is
distinct from the C2PA hard binding, which validates the final signed asset
bytes and detects post-signing tampering.

**Type:** `string` | **Required:** Yes

### `label`

Stencila label associated with the asset.

Labels such as `fig:example` are how authors and readers often refer to
outputs in a document. They are optional because not every signed asset is
a labelled Stencila node.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `title`

Human-readable title for the asset.

This is display metadata for reviewers. It is optional and should not be
treated as a stable identifier because titles can change independently of
the underlying bytes.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `size`

Asset byte length before signing.

Size is a cheap consistency check and useful in audit output. It is
optional because some producers may stream content without retaining a
length at projection time.

**Type:** `integer` | **Required:** No | **Nullable:** Yes

### `width`

Width for image or video assets.

Dimensions help consumers understand the signed rendition without opening
the asset. Width is optional because it only applies to some media types.

**Type:** `integer` | **Required:** No | **Nullable:** Yes

### `height`

Height for image or video assets.

Dimensions are recorded as unsigned integers in rendered pixel units
unless the media type defines a different native unit.

**Type:** `integer` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
