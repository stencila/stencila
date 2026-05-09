---
title: "Verification Record"
description: "Reproducibility status and comparison details known when signing."
---

# Verification Record

Reproducibility status and comparison details known when signing.

This record reports what Stencila knew at signing time. It is not the same as
C2PA manifest validation, which verifies signatures and asset binding.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`reproducibilityStatus`](#reproducibility-status) | `string` | Yes | Reproducibility status. |
| [`policy`](#policy) | `string` | No | Verification policy used. |
| [`verifiedBy`](#verified-by) | `string` | No | Verifier identity. |
| [`verifiedAt`](#verified-at) | `string` | No | Verification timestamp in RFC 3339 format. |
| [`comparison`](#comparison) | any JSON value | No | Structured comparison details. |

### `reproducibilityStatus`

Reproducibility status.

Suggested values are `not-checked`, `reproduced`, `changed`, `failed`,
and `not-reproducible`. The field is required so absence never masks an
intentionally skipped reproducibility check.

**Type:** `string` | **Required:** Yes

### `policy`

Verification policy used.

Policy explains how strict the comparison was, which outputs were
compared, and what tolerances applied. It is optional because initial
signing often records `not-checked`.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `verifiedBy`

Verifier identity.

This can be a person, service, or software identifier. Rich verifier
attribution should also be represented in `attributions` when relevant.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `verifiedAt`

Verification timestamp in RFC 3339 format.

The timestamp records when reproducibility was checked, not when the C2PA
claim was signed.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `comparison`

Structured comparison details.

This intentionally remains JSON so comparison tools can include
domain-specific summaries without forcing every detail into v1.

**Type:** any JSON value | **Required:** No


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
