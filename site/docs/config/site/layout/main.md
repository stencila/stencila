---
title: Site Layout Main Config
description: Main content area configuration
---

Main content area configuration

Controls formatting of the main content area including content width,
padding, and title visibility. These properties are orthogonal to the
structural layout (regions, sidebars, etc.).

```toml
[site.layout.main]
width = "none"
padding = "none"
title = false
```

**Type:** `MainConfig`

# `width`

**Type:** `ContentWidth` (optional)

Maximum width for content elements

Controls `max-width` on content children (paragraphs, headings, etc.).
Defaults to `65ch` for optimal reading line length.
Set to `"none"` for full-width content.

# `padding`

**Type:** `ContentPadding` (optional)

Padding around the main content area

Controls padding on the `#main-content` element. Defaults to theme
content spacing. Set to `"none"` for full-bleed content.

# `title`

**Type:** `boolean` (optional)

Whether to display the document title slot

When `false`, the `[slot="title"]` section is hidden via CSS.
The HTML `<title>` element is unaffected (preserving SEO).
Defaults to `true`.


***

This documentation was generated from [`layout/config.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/config.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
