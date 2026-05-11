---
title: "Disclosure Assessment Record"
description: "Assessment of whether a class of sensitive data is present."
---

# Disclosure Assessment Record

Assessment of whether a class of sensitive data is present.

This small record is used instead of a boolean because signed privacy claims
should be explicit about whether the data was assessed, redacted, detected,
not detected, or intentionally undisclosed.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`status`](#status) | `string` | Yes | Assessment status. |
| [`policy`](#policy) | `string` | No | Policy, scanner, or rule set used for the assessment. |
| [`assessedAt`](#assessed-at) | `string` | No | Assessment timestamp in RFC 3339 format. |

### `status`

Assessment status.

Suggested values are `not-assessed`, `none-detected`, `detected`,
`redacted`, and `undisclosed`. The field is required so consumers can
avoid interpreting absence as a negative claim.

**Type:** `string` | **Required:** Yes

### `policy`

Policy, scanner, or rule set used for the assessment.

Policy is optional because not all producers have a named scanner or
policy at signing time.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `assessedAt`

Assessment timestamp in RFC 3339 format.

This records when the assessment was made, which can differ from signing
and reproducibility verification times.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
