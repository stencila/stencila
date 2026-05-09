---
title: "AI Disclosure Record"
description: "Stencila projection of C2PA AI disclosure concepts."
---

# AI Disclosure Record

Stencila projection of C2PA AI disclosure concepts.

This record keeps Stencila-specific provenance summaries connected to the AI
transparency vocabulary used by C2PA. It mirrors the standard field names
where they are stable, but remains a local projection. Producers that
disclose AI model use should also emit the standard `c2pa.ai-disclosure`
assertion when possible.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`modelType`](#model-type) | `string` | Yes | C2PA AI model type. |
| [`modelName`](#model-name) | `string` | No | Human-readable model name. |
| [`modelIdentifier`](#model-identifier) | `string` | No | Unique model identifier, URI, or package URL. |
| [`contentProfile`](#content-profile) | [`AiContentProfileRecord`](ai-content-profile-record) | No | Content profile for the AI use. |
| [`scientificDomain`](#scientific-domain) | array<`string`> | No | Scientific domains associated with the AI use. |
| [`standardAssertion`](#standard-assertion) | `string` | No | Label or URI of the corresponding standard C2PA assertion. |

### `modelType`

C2PA AI model type.

This mirrors the standard AI disclosure `modelType` field so Stencila
consumers can reason about the model class without parsing another
assertion.

**Type:** `string` | **Required:** Yes

### `modelName`

Human-readable model name.

The name is display metadata. Durable identity should use
`modelIdentifier` where available.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `modelIdentifier`

Unique model identifier, URI, or package URL.

This mirrors the standard AI disclosure `modelIdentifier` field and is
preferred for reproducibility and compliance checks.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `contentProfile`

Content profile for the AI use.

This mirrors the standard AI disclosure `contentProfile` object for
structured content-context and human-oversight signals.

**Type:** [`AiContentProfileRecord`](ai-content-profile-record) | **Required:** No | **Nullable:** Yes

### `scientificDomain`

Scientific domains associated with the AI use.

Values should follow the C2PA AI disclosure convention, such as arXiv
taxonomy strings, when disclosed.

**Type:** array<`string`> | **Required:** No

### `standardAssertion`

Label or URI of the corresponding standard C2PA assertion.

This optional pointer prevents the Stencila projection from drifting away
from the standard assertion when both are present.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
