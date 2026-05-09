---
title: "Dependency Record"
description: "Dependency of an execution."
---

# Dependency Record

Dependency of an execution.

Dependencies represent the specific upstream nodes, values, or artifacts that
Stencila knew the execution used. They complement the aggregate dependency
digest on `DocumentRecord`.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`nodeId`](#node-id) | `string` | No | Depended-on node identifier. |
| [`nodeType`](#node-type) | `string` | No | Depended-on Stencila node type. |
| [`relation`](#relation) | `string` | No | Dependency relation. |
| [`digest`](#digest) | `string` | No | Digest of the depended-on node or value. |

### `nodeId`

Depended-on node identifier.

Node IDs let verifiers connect dependency facts back to a Stencila
document tree when that tree is available.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `nodeType`

Depended-on Stencila node type.

The node type helps consumers understand whether the dependency was code,
data, prose, a parameter, or another output.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `relation`

Dependency relation.

Suggested values are `input`, `output`, `state`, and `parameter`; custom
values should use a reverse-DNS prefix when they are not Stencila terms.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `digest`

Digest of the depended-on node or value.

The value should use `algorithm:hex` form. It is optional because some
dependencies are intentionally disclosed only by identity or relation.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
