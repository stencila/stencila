---
title: Site Layout Responsive Config
description: Global responsive configuration for sidebar collapse
---

Global responsive configuration for sidebar collapse

These settings apply to both sidebars unless overridden per-sidebar.

Example:
```toml
[site.layout.responsive]
breakpoint = 1024
toggle-style = "fixed-edge"
```

**Type:** `ResponsiveConfig`

# `breakpoint`

**Type:** `integer` (optional)

Breakpoint at which sidebars collapse (in pixels)

Default: 1024

# `collapsible`

**Type:** `boolean` (optional)

Whether the sidebar is collapsible

Default: true

# `toggle-style`

**Type:** `SidebarToggleStyle` (optional)

Toggle button style

Default: fixed-edge

| Value | Description |
|-------|-------------|
| `fixed-edge` | Fixed edge buttons (buttons fixed to left/right viewport edges) |
| `header` | Header buttons (toggle buttons inside header region) |
| `hamburger` | Hamburger menu (single button for all sidebars) |


***

This documentation was generated from [`layout/config.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/config.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
