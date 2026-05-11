---
title: "Execution Message Record"
description: "Message emitted during execution."
---

# Execution Message Record

Message emitted during execution.

Messages provide trust-relevant warnings and errors while avoiding full log
capture. They should be filtered under the privacy policy before signing.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`level`](#level) | `string` | No | Message severity. |
| [`errorType`](#error-type) | `string` | No | Error type or class, for error messages. |
| [`message`](#message) | `string` | No | Human-readable message text. |

### `level`

Message severity.

Values should align with Stencila `MessageLevel`, for example `Info`,
`Warning`, or `Error`.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `errorType`

Error type or class, for error messages.

This supports automated triage without requiring consumers to parse the
human-readable message text.

**Type:** `string` | **Required:** No | **Nullable:** Yes

### `message`

Human-readable message text.

Message text is optional and should be redacted when it contains source
code, secrets, private paths, or other sensitive details.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
