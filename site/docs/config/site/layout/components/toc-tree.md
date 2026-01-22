---
title: Site Layout TocTree Component
description: Table of contents tree from document headings
---

Table of contents tree from document headings

Generates a hierarchical list of links from the current page's headings.
Includes scroll-spy that highlights the currently visible section as
the user scrolls. Renders nothing if the page has no headings.

```toml
[site.layout.right-sidebar]
start = "toc-tree"  # Uses defaults

# With custom title and deeper depth:
start = { type = "toc-tree", title = "Contents", depth = 4 }
```

## `title`

**Type:** `string` (optional)

Title above the TOC

Default: "On this page"

## `depth`

**Type:** `integer` (optional)

Maximum heading depth to include

Controls which heading levels appear in the TOC. For example,
depth=3 includes h1, h2, and h3 headings.

Default: 3


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
