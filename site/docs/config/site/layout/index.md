---
title: Site Layout Config
description: Site layout configuration
---

Site layout configuration

Controls the layout structure of site pages using a region-based system.
Each region (header, sidebars, etc.) has sub-regions (start, middle, end)
where components can be placed.

Example:
```toml
[site.layout]
preset = "docs"

[site.layout.header]
start = "logo"
end = ["icon-links", "color-mode"]
```

# `preset`

**Type:** `LayoutPreset` (optional)

Named preset to use as base (docs, blog, landing, api)

Presets provide sensible defaults that can be extended with explicit config.

| Value | Description |
|-------|-------------|
| `docs` | Documentation site: nav-tree left, toc-tree right, breadcrumbs, prev-next |
| `blog` | Blog/article site: no left sidebar, toc-tree right, no prev-next |
| `landing` | Landing page: no sidebars, centered content |
| `api` | API reference: nav-tree left (flat), no right sidebar |

# `header`

**Type:** [`RegionConfig`](header.md) (optional)

Header region configuration

# `left-sidebar`

**Type:** [`RegionConfig`](left-sidebar.md) (optional)

Left sidebar region configuration

# `top`

**Type:** [`RegionConfig`](top.md) (optional)

Top region configuration

# `bottom`

**Type:** [`RegionConfig`](bottom.md) (optional)

Bottom region configuration

# `right-sidebar`

**Type:** [`RegionConfig`](right-sidebar.md) (optional)

Right sidebar region configuration

# `footer`

**Type:** [`RegionConfig`](footer.md) (optional)

Footer region configuration

# `responsive`

**Type:** [`ResponsiveConfig`](responsive.md) (optional)

Global responsive configuration for sidebar collapse

# `components`

**Type:** [`components`](components/)

Named component definitions for reuse

# `overrides`

**Type:** `array`

Route-specific layout overrides

First matching override wins (order matters).

Example:
```toml
[[site.layout.overrides]]
routes = ["/blog/**"]
left-sidebar = false
```


***

This documentation was generated from [`layout/config.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/config.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
