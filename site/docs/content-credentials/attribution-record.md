---
title: "Attribution Record"
description: "Role-bearing attribution for an agent."
---

# Attribution Record

Role-bearing attribution for an agent.

This is the public, compact equivalent of Stencila `AuthorRole`. It records
the contributing agent, the role they played, optional Stencila provenance
counts, and the scope of the attribution.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`id`](#id) | `string` | No | Optional attribution identifier. |
| [`agent`](#agent) | [`AgentRecord`](agent-record) | Yes | Agent credited or responsible for the role. |
| [`roleName`](#role-name) | `string` | No | Stencila `AuthorRoleName` or compatible role value. |
| [`format`](#format) | `string` | No | Format in which the role was performed. |
| [`lastModified`](#last-modified) | `string` | No | Most recent modification time by this agent in this role. |
| [`scope`](#scope) | `string` | No | Scope of the attribution. |
| [`provenanceCategory`](#provenance-category) | `string` | No | Stencila provenance category associated with this attribution. |
| [`characterCount`](#character-count) | `integer` | No | Number of characters attributed to this agent and role. |
| [`characterPercent`](#character-percent) | `number` | No | Percentage of counted content attributed to this agent and role. |

### `id`

Optional attribution identifier.

Activity records refer to associated agents through this ID. It is
optional so simple assertions can still list attributions without
inventing local identifiers.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `agent`

Agent credited or responsible for the role.

The agent can be a person, organization, software application, model, or
intentionally anonymous thing. Using a common agent record keeps human and
machine authorship in the same Stencila-compatible attribution model.

**Type:** [`AgentRecord`](agent-record) | **Required:** Yes

### `roleName`

Stencila `AuthorRoleName` or compatible role value.

Known values include `Writer`, `Verifier`, `Instructor`, `Generator`, and
`Executor`. The field is optional because flat Stencila `Author` values
and external metadata can identify an author without a known role.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `format`

Format in which the role was performed.

This mirrors `AuthorRole.format`, for example Markdown, Python, or HTML.
It helps distinguish writing source code from editing rendered output.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `lastModified`

Most recent modification time by this agent in this role.

The timestamp should be RFC 3339. It is optional because privacy profiles
may strip timestamps and because not all author metadata tracks last
modification.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `scope`

Scope of the attribution.

Suggested values are `document`, `asset`, `activity`, `input`, and
`output`. The scope prevents a workflow executor from being mistaken for
a bibliographic author of the whole work.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `provenanceCategory`

Stencila provenance category associated with this attribution.

Values come from `ProvenanceCategory`, such as `Hw`, `Mw`, or `MwHeHv`.
The category is optional because authorship roles and provenance counts
are related but not identical concepts.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `characterCount`

Number of characters attributed to this agent and role.

Character counts are optional because they only apply to text-like
content and may be redacted for privacy or size reasons.

**Type:** `integer` | **Required:** No | **Nullable:** Yes

### `characterPercent`

Percentage of counted content attributed to this agent and role.

This is a 0-100 percentage, matching Stencila `ProvenanceCount` semantics.
It is optional because counts may be available without a computed percent.

**Type:** `number` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
