---
title: "Runtime Record"
description: "Runtime version relevant to reproduction."
---

# Runtime Record

Runtime version relevant to reproduction.

Runtime records cover language runtimes, package managers, database engines,
or other executable environments that materially affect outputs.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`name`](#name) | `string` | No | Runtime name. |
| [`version`](#version) | `string` | No | Runtime version. |

### `name`

Runtime name.

Examples include `python`, `r`, `node`, `quarto`, or a package manager
name. It is optional for redacted or partially known environments.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `version`

Runtime version.

Version is optional because some runtimes are implicit in a container or
unavailable from the execution backend.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
