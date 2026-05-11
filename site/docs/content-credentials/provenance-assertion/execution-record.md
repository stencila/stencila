---
title: "Execution Record"
description: "Execution facts for executable document nodes."
---

# Execution Record

Execution facts for executable document nodes.

This record mirrors Stencila execution state. It is narrower than `activity`
and should be present when the represented asset depends on executing code,
a prompt, a query, or another executable document node.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`status`](#status) | `string` | No | Execution status reported by Stencila. |
| [`endedAt`](#ended-at) | `string` | No | Execution end time in RFC 3339 format. |
| [`durationMs`](#duration-ms) | `integer` | No | Execution duration in milliseconds. |
| [`digests`](#digests) | [`ExecutionDigestsRecord`](execution-digests-record) | No | Digests representing executable node state at signing time. |
| [`count`](#count) | `integer` | No | Kernel execution counter for the node. |
| [`kernel`](#kernel) | [`KernelRecord`](kernel-record) | No | Kernel used to execute the node. |
| [`dependencies`](#dependencies) | array<[`DependencyRecord`](dependency-record)> | No | Other document nodes or values this execution depended on. |
| [`messages`](#messages) | array<[`ExecutionMessageRecord`](execution-message-record)> | No | Execution messages emitted while producing the asset. |

### `status`

Execution status reported by Stencila.

Values should come from Stencila `ExecutionStatus` when possible, for
example `Succeeded`, `Failed`, or `Skipped`. Extension strings are allowed
for future statuses.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `endedAt`

Execution end time in RFC 3339 format.

The field is named `endedAt` to match the activity timing vocabulary.
The legacy `ended` name is accepted for unpublished pre-v1 payloads.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `durationMs`

Execution duration in milliseconds.

Milliseconds give enough precision for user-facing diagnostics without
encoding a language-specific duration format.

**Type:** `integer` | **Required:** No | **Nullable:** Yes

### `digests`

Digests representing executable node state at signing time.

These compactly attest the state that Stencila considered relevant to
generated output, without disclosing the source code or dependency values
themselves. The field is named `digests` because it is already nested
under `execution`.

**Type:** [`ExecutionDigestsRecord`](execution-digests-record) | **Required:** No | **Nullable:** Yes

### `count`

Kernel execution counter for the node.

This preserves notebook-style execution context for reviewers. It is
optional because many kernels or workflows do not expose a counter.

**Type:** `integer` | **Required:** No | **Nullable:** Yes

### `kernel`

Kernel used to execute the node.

Kernel identity is separate from environment runtime versions because a
named kernel can wrap a language runtime, container, or remote service.

**Type:** [`KernelRecord`](kernel-record) | **Required:** No | **Nullable:** Yes

### `dependencies`

Other document nodes or values this execution depended on.

Dependencies explain why an output should be rerun when upstream state
changes. They can be redacted or summarized independently of aggregate
digest values on this execution record.

**Type:** array<[`DependencyRecord`](dependency-record)> | **Required:** No

### `messages`

Execution messages emitted while producing the asset.

Messages are kept compact because manifests should not become log files.
They are included to preserve warnings or errors that materially affect
trust in the output.

**Type:** array<[`ExecutionMessageRecord`](execution-message-record)> | **Required:** No


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
