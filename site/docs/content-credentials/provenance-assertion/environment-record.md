---
title: "Environment Record"
description: "Environment facts selected for publication under the active privacy profile."
---

# Environment Record

Environment facts selected for publication under the active privacy profile.

The environment record captures reproducibility context without attempting to
be a full software bill of materials. It keeps the assertion compact and
privacy-aware while still surfacing the facts most likely to affect reruns.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`containerImage`](#container-image) | `string` | No | Container image used for execution. |
| [`os`](#os) | `string` | No | Operating system name or identifier. |
| [`architecture`](#architecture) | `string` | No | CPU architecture. |
| [`runtimes`](#runtimes) | array<[`RuntimeRecord`](runtime-record)> | No | Runtime versions relevant to reproduction. |
| [`manifests`](#manifests) | array<[`FileDigestRecord`](file-digest-record)> | No | Environment manifest digests relevant to reproduction. |
| [`lockfiles`](#lockfiles) | array<[`FileDigestRecord`](file-digest-record)> | No | Lockfile digests relevant to reproduction. |
| [`repository`](#repository) | `string` | No | Source repository URL or identifier for the environment context. |
| [`commit`](#commit) | `string` | No | Source commit hash for the environment context. |
| [`informationalUri`](#informational-uri) | `string` | No | Informational URI for locating the environment context. |

### `containerImage`

Container image used for execution.

A container image is often the strongest environment identity, but it is
optional because many local or hosted executions do not use containers or
cannot disclose image names.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `os`

Operating system name or identifier.

OS details are useful for reproduction but can reveal infrastructure, so
they should be disclosed only under an appropriate privacy policy.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `architecture`

CPU architecture.

Architecture can affect binary packages and numeric results. It is kept
separate from OS because cross-platform containers can mix the two.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `runtimes`

Runtime versions relevant to reproduction.

Runtime records are intentionally lightweight and ordered by the producer
so a verifier can display the most important runtimes first.

**Type:** array<[`RuntimeRecord`](runtime-record)> | **Required:** No

### `manifests`

Environment manifest digests relevant to reproduction.

Manifests identify declared dependency and tool requirements, complementing
lockfiles that identify resolved dependency state.

**Type:** array<[`FileDigestRecord`](file-digest-record)> | **Required:** No

### `lockfiles`

Lockfile digests relevant to reproduction.

Digests identify dependency state without embedding potentially large or
private lockfiles in the manifest.

**Type:** array<[`FileDigestRecord`](file-digest-record)> | **Required:** No

### `repository`

Source repository URL or identifier for the environment context.

This usually matches the source repository, but is stored with the
environment so the manifest and lockfile paths can be resolved even when
the signed asset does not disclose a source document.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `commit`

Source commit hash for the environment context.

A full commit hash lets verifiers locate immutable manifest and lockfile
contents when the repository URL is public.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `informationalUri`

Informational URI for locating the environment context.

This should point to immutable, human-browsable environment context such
as a repository tree at a full commit hash.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
