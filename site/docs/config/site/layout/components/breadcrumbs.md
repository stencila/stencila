---
title: Site Layout Breadcrumbs Component
description: Breadcrumb navigation trail
---

Breadcrumb navigation trail

Displays the hierarchical path from site root to the current page.
Each path segment is converted to title case (e.g., "getting-started" →
"Getting Started"). Intermediate segments are clickable links if the
route exists; the current page is shown as non-clickable text.

This component has no configurable options.

```toml
[site.layout.top]
start = "breadcrumbs"

[site.layout.header]
middle = "breadcrumbs"
```


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
