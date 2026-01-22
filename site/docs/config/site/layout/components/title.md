---
title: Site Layout Title Component
description: Site title text
---

Site title text

Displays the site title as text. When used as a bare `"title"` string,
uses the value from `site.title`. When `text` is specified, it overrides
`site.title` for this instance only.

```toml
[site.layout.header]
start = ["logo", "title"]  # Uses site.title

# Override for this component only:
start = [{ type = "title", text = "Docs" }]
```

## `text`

**Type:** `string` (optional)

Title text (defaults to site.title)

When not specified, inherits from `site.title`. If both are empty,
the component renders nothing.


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
