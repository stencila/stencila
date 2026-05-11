---
title: "AI Content Profile Record"
description: "Content profile for AI disclosure."
---

# AI Content Profile Record

Content profile for AI disclosure.

The record mirrors the standard C2PA AI disclosure `contentProfile` object
while allowing Stencila to preserve unknown future profile fields.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| [`humanOversightLevel`](#human-oversight-level) | `string` | No | Human oversight level. |

### `humanOversightLevel`

Human oversight level.

Values should follow C2PA AI disclosure, for example
`fully_autonomous`, `prompt_guided`, or `human_validated`.

**Type:** `string` | **Required:** No | **Nullable:** Yes


---

This documentation was generated from [`schema.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/schema.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/content-credentials/src/bin/generate.rs).
