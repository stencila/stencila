---
title: "Definition Record"
description: "Definition loaded by a workflow."
---

# Definition Record

Definition loaded by a workflow.

Definitions represent workflow, agent, skill, policy, or other reusable
resources that may affect generated outputs. The `definitionType`, `name`, `role`,
and `contentDigest` fields are sufficient to link a record back to the
Stencila workspace definition tables when `runId` is also disclosed.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`definitionType`](#definition-type) | `string` | No | Definition type. |
| [`name`](#name) | `string` | No | Definition name. |
| [`role`](#role) | `string` | No | Role this definition played in the workflow run. |
| [`sourcePath`](#source-path) | `string` | No | Source path for the definition. |
| [`version`](#version) | `string` | No | Definition version. |
| [`contentDigest`](#content-digest) | `string` | No | Digest of definition content. |

### `definitionType`

Definition type.

Type distinguishes workspace definition snapshots such as `workflow`,
`agent`, and `skill` without requiring a separate record for each class.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `name`

Definition name.

Names help audit readability but are optional because they can reveal
private project structure.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `role`

Role this definition played in the workflow run.

Role corresponds to `workflow_run_definitions.role`. It lets consumers
disambiguate how a content-addressed definition snapshot contributed to
the run, for example as a workflow, agent, skill, or policy.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `sourcePath`

Source path for the definition.

Paths are useful for reproduction but often private, so this field should
be set only when the active privacy policy permits it.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `version`

Definition version.

Version complements the content digest when definitions are distributed
as named packages or managed resources.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `contentDigest`

Digest of definition content.

The value should use `algorithm:hex` form. It is preferred over raw
content so private definitions can still be attested.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
