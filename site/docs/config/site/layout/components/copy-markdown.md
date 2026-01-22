---
title: Site Layout CopyMarkdown Component
description: Copy page as Markdown button
---

Copy page as Markdown button

Displays a button that copies the current page content as Markdown
to the clipboard. Requires `formats = ["md"]` in site config (the default).
The markdown is fetched from `page.md` which is generated during site build.

```toml
[site.layout.footer]
end = "copy-markdown"  # Default text: "Copy as Markdown"

# Custom text:
end = { type = "copy-markdown", text = "Copy as MD" }

# Icon only:
end = { type = "copy-markdown", style = "icon" }
```

## `text`

**Type:** `string` (optional)

Custom button text

Default: "Copy as Markdown"

## `style`

**Type:** `CopyMarkdownStyle` (optional)

Display style

Default: both

| Value | Description |
|-------|-------------|
| `icon` | Clipboard icon only |
| `text` | Text only |
| `both` | Icon and text (default) |


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
