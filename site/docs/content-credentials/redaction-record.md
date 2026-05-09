---
title: "Redaction Record"
description: "Redaction applied to a field while projecting the assertion."
---

# Redaction Record

Redaction applied to a field while projecting the assertion.

Redaction records are intentionally compact so privacy-relevant decisions can
be audited without exposing the redacted value.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`field`](#field) | `string` | No | Redacted field path. |
| [`reason`](#reason) | `string` | No | Reason the field was redacted. |

### `field`

Redacted field path.

Paths should use assertion JSON field names, for example
`workflow.goalDigest` or `inputs[0].uri`, so consumers can locate the
omitted context in the signed payload.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `reason`

Reason the field was redacted.

Suggested values include `personal-data`, `secret`, `private-path`,
`policy`, and `size`. Custom values should be stable strings rather than
user-facing prose.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
