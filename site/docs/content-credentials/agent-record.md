---
title: "Agent Record"
description: "Agent participating in provenance."
---

# Agent Record

Agent participating in provenance.

The same record is used for human, organizational, software, and model
agents. That keeps the schema aligned with Stencila `Author` and `AuthorRole`
while also mapping cleanly to C2PA's broader notion of actors.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`kind`](#kind) | `string` | No | Agent kind. |
| [`name`](#name) | `string` | No | Agent name. |
| [`id`](#id) | `string` | No | Stable local or external agent identifier. |
| [`identifiers`](#identifiers) | array<[`IdentifierRecord`](identifier-record)> | No | Structured identifiers for the agent. |
| [`provider`](#provider) | `string` | No | Organization, service, or model provider. |
| [`version`](#version) | `string` | No | Software or model version. |
| [`model`](#model) | `string` | No | Model name used by an AI agent. |
| [`modelIdentifier`](#model-identifier) | `string` | No | Durable model identifier, URI, or package URL. |
| [`url`](#url) | `string` | No | Agent URL. |

### `kind`

Agent kind.

Suggested values are `person`, `organization`, `softwareApplication`,
`model`, and `thing`. It is optional so redacted or legacy agents can
still be represented by name or identifier alone.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `name`

Agent name.

Names are display metadata and may be pseudonymous or redacted. Stable
identity should be represented in `identifiers` where possible.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `id`

Stable local or external agent identifier.

This field is for a primary identifier that other records may reference.
Additional identity schemes belong in `identifiers`.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `identifiers`

Structured identifiers for the agent.

This supports ORCID for people, ROR for organizations, package URLs for
software, and model identifiers without adding a top-level field for each
identity scheme.

**Type:** array<[`IdentifierRecord`](identifier-record)> | **Required:** No

### `provider`

Organization, service, or model provider.

Provider is optional because not all agents are mediated by a provider,
but it is useful for AI and remote execution agents where model names are
not globally unique.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `version`

Software or model version.

Version is kept on the agent because generator and executor software can
be credited as authors independently of the Stencila producer version.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `model`

Model name used by an AI agent.

This remains optional and display-oriented. Durable model identity should
use `modelIdentifier` or a typed identifier.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `modelIdentifier`

Durable model identifier, URI, or package URL.

Keeping this separate from `model` allows a readable name and a stable
machine identifier to coexist.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `url`

Agent URL.

URLs are useful for software and organization records but optional
because they can expose private infrastructure or become stale.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
