---
title: Site Layout Components
description: Named component definitions for reuse
---

Named component definitions for reuse

Define components once and reference them by name in regions.

Example:
```toml
[site.layout.components.main-nav]
type = "nav-tree"
collapsible = true
depth = 3
```

## Available Types

| Type | Description |
|------|-------------|
| [`logo`](logo.md) | Site logo image with responsive and dark mode variants |
| [`title`](title.md) | Site title text |
| [`breadcrumbs`](breadcrumbs.md) | Breadcrumb navigation trail |
| [`nav-tree`](nav-tree.md) | Hierarchical navigation tree |
| [`nav-menu`](nav-menu.md) | Top-level navigation menu bar |
| [`nav-groups`](nav-groups.md) | Footer-style grouped navigation |
| [`toc-tree`](toc-tree.md) | Table of contents tree from document headings |
| [`prev-next`](prev-next.md) | Previous/next page navigation links |
| [`color-mode`](color-mode.md) | Light/dark mode toggle |
| [`copyright`](copyright.md) | Copyright notice with auto-updating year |
| [`social-links`](social-links.md) | Social/external links (GitHub, Discord, LinkedIn, etc.) |
| [`edit-source`](edit-source.md) | Edit source link for GitHub/GitLab/Bitbucket |
| [`edit-on`](edit-on.md) | Edit on cloud service (Google Docs or Microsoft 365) |
| [`copy-markdown`](copy-markdown.md) | Copy page as Markdown button |


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
