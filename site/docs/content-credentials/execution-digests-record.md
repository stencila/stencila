---
title: "Execution Digests Record"
description: "Digest values corresponding to Stencila `CompilationDigest`."
---

# Execution Digests Record

Digest values corresponding to Stencila `CompilationDigest`.

Stencila distinguishes state, semantic content, and dependency digests so a
verifier can understand why an output might be stale without seeing the full
source. This record preserves that distinction in the public assertion.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`state`](#state) | `string` | No | Digest of execution state that affects generated output. |
| [`semantic`](#semantic) | `string` | No | Digest of semantic content that affects generated output. |
| [`dependencies`](#dependencies) | `string` | No | Digest of dependencies that affect generated output. |
| [`dependenciesStale`](#dependencies-stale) | `integer` | No | Number of stale dependencies known at signing time. |
| [`dependenciesFailed`](#dependencies-failed) | `integer` | No | Number of failed dependencies known at signing time. |

### `state`

Digest of execution state that affects generated output.

This usually covers code, parameter values, and other state needed to
decide whether an executable node should be rerun.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `semantic`

Digest of semantic content that affects generated output.

Semantic digests let Stencila distinguish meaningful source changes from
formatting or metadata churn.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `dependencies`

Digest of dependencies that affect generated output.

This summarizes upstream nodes or values used by the executable node. It
is separate from `dependencies` because the digest is compact and stable
even when individual dependencies are redacted.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `dependenciesStale`

Number of stale dependencies known at signing time.

A nonzero value warns verifiers that the output may not reflect current
upstream state even if the signed asset itself is valid.

**Type:** `integer` | **Required:** No | **Nullable:** Yes

### `dependenciesFailed`

Number of failed dependencies known at signing time.

Failed dependencies are recorded separately from stale dependencies
because they indicate an execution failure path rather than merely an
out-of-date one.

**Type:** `integer` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
