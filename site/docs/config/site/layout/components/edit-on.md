---
title: Site Layout EditOn Component
description: Edit on cloud service (Google Docs or Microsoft 365)
---

Edit on cloud service (Google Docs or Microsoft 365)

Displays a link to edit the current page on Google Docs or Microsoft 365
via Stencila Cloud. Only renders if `workspace.id` is configured, e.g.

```toml
# Edit on cloud service
[site.layout.footer]
end = "edit-on:gdocs"  # Edit on Google Docs
# or
end = "edit-on:m365"   # Edit on Microsoft 365

# With custom text:
end = { type = "edit-on", service = "gdocs", text = "Open in Google Docs" }
```

## `service`

**Type:** `EditOnService`

Cloud service to edit on (gdocs or m365)

| Value | Description |
|-------|-------------|
| `gdocs` | Google Docs |
| `m365` | Microsoft 365 |

## `text`

**Type:** `string` (optional)

Custom link text

Default: "Edit on Google Docs" or "Edit on Microsoft 365"

## `style`

**Type:** `EditSourceStyle` (optional)

Display style

Default: both

| Value | Description |
|-------|-------------|
| `icon` | Pencil/edit icon only |
| `text` | Text only |
| `both` | Icon and text (default) |


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
