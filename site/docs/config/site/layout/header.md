---
title: Site Layout Header Config
description: Header region configuration
---

Header region configuration

Horizontal bar at the top of every page, spanning the full width.
Sub-regions flow left-to-right: start | middle | end.

Typical components: logo, title, nav-menu, social-links, color-mode.

```toml
[site.layout.header]
start = "logo"
middle = "nav-menu"
end = ["social-links", "color-mode"]
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


***

This documentation was generated from [`layout/regions.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/regions.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
