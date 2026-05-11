---
title: "Identifier Record"
description: "Identifier for an agent or other named entity."
---

# Identifier Record

Identifier for an agent or other named entity.

A small typed identifier record is more future-proof than adding separate
optional fields for every identity scheme that Stencila may later support.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`identifierType`](#identifier-type) | `string` | No | Identifier type or scheme. |
| [`value`](#value) | `string` | No | Identifier value. |

### `identifierType`

Identifier type or scheme.

Examples include `orcid`, `ror`, `url`, `purl`, `doi`, and `modelId`.
Consumers should treat unknown types as opaque labels.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `value`

Identifier value.

The value is a string because identity schemes differ in syntax and some
use URIs while others use compact IDs.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
