---
title: "Kernel Record"
description: "Kernel used to execute a Stencila node."
---

# Kernel Record

Kernel used to execute a Stencila node.

Kernels are separated from runtimes because the same language runtime can be
exposed through different kernels with different execution semantics.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`name`](#name) | `string` | No | Kernel name. |
| [`version`](#version) | `string` | No | Kernel version. |
| [`language`](#language) | `string` | No | Programming language handled by the kernel. |

### `name`

Kernel name.

The name should be stable enough for a human to identify the execution
backend, for example `python`, `r`, or a provider-specific kernel name.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `version`

Kernel version.

Version is optional because some kernels are wrappers around runtimes
whose exact version is better captured in `environment.runtimes`.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `language`

Programming language handled by the kernel.

Language helps verifiers select reproduction tooling even when the kernel
name is provider-specific.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
