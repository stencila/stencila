---
title: Site Layout Left Sidebar Config
description: Left sidebar region configuration
---

Left sidebar region configuration

Vertical sidebar on the left side of the page.
Sub-regions flow top-to-bottom: start | middle | end.
Auto-enabled for multi-page sites when not specified.

Typical components: nav-tree.

```toml
[site.layout.left-sidebar]
start = { type = "nav-tree", collapsible = true }
```

**Type:** `RegionConfig`

# `enabled`

**Type:** `boolean` (optional)

Explicit enable/disable (for use in overrides that also set sub-regions)

# `start`

**Type:** `array` (optional)

Components in the start sub-region (left for horizontal, top for vertical)

# `middle`

**Type:** `array` (optional)

Components in the middle sub-region (center)

# `end`

**Type:** `array` (optional)

Components in the end sub-region (right for horizontal, bottom for vertical)

# `rows`

**Type:** `array` (optional)

Multiple rows, each with their own start/middle/end sub-regions

When specified, `start`, `middle`, and `end` are ignored and each row
is rendered separately. This enables multi-row layouts within a region.

Example (applicable to any region):
```toml
rows = [
  { middle = "prev-next" },
  { start = "edit-source", end = "copyright" }
]
```

# `responsive`

**Type:** [`ResponsiveConfig`](./responsive.md) (optional)

Responsive configuration (only applicable to sidebars)

Controls when the sidebar becomes collapsible and how the toggle appears.


***

This documentation was generated from [`layout/regions.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/regions.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
