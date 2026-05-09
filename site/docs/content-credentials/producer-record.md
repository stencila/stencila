---
title: "Producer Record"
description: "Producer metadata embedded in the assertion."
---

# Producer Record

Producer metadata embedded in the assertion.

This describes the Stencila software component acting as C2PA claim
generator. It is intentionally narrower than `AgentRecord`: a producer is
about the mechanism that made the manifest, not about authorship credit.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`name`](#name) | `string` | Yes | Producer name. |
| [`version`](#version) | `string` | Yes | Producer software version. |
| [`stencilaSchemaVersion`](#stencila-schema-version) | `string` | No | Stencila Schema version used while producing the asset. |
| [`codec`](#codec) | `string` | No | Codec used to encode or export the signed asset. |
| [`renderer`](#renderer) | `string` | No | Renderer or application component that produced the asset bytes. |

### `name`

Producer name.

This is usually `Stencila`. It is required so a generic manifest consumer
can display who generated the custom assertion even if it ignores the
nested Stencila-specific records.

**Type:** `string` | **Required:** Yes

### `version`

Producer software version.

Version is required because signed payloads are immutable. When a schema
interpretation or privacy projection bug is discovered later, the
producer version is the quickest way to scope affected manifests.

**Type:** `string` | **Required:** Yes

### `stencilaSchemaVersion`

Stencila Schema version used while producing the asset.

This is separate from the assertion payload `version`. It records the
document node vocabulary that supplied values such as `nodeType`,
`AuthorRoleName`, and `ProvenanceCategory`.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `codec`

Codec used to encode or export the signed asset.

The codec explains the transformation from Stencila document state to
bytes. It is optional because some manually signed assets have no Stencila
codec involved.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `renderer`

Renderer or application component that produced the asset bytes.

Rendering can be independent of the codec, for example a CLI command,
web component, or browser engine. Recording it helps reproduce visual
outputs without overloading `producer.name`.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
