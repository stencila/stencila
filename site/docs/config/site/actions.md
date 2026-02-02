---
title: Site Actions Config
description: Site actions zone configuration
---

Site actions zone configuration

Controls the position, direction, and mode of floating action buttons (FABs)
like reviews and uploads. All actions share a unified position on the page.

```toml
# Configure the actions zone
[site.actions]
position = "bottom-right"  # Corner position (default)
direction = "vertical"     # Stack direction (default)
mode = "collapsed"         # Display mode (default)
```

**Type:** `SiteActionsConfig`

# `position`

**Type:** `SiteActionsPosition` (optional)

Position of the actions zone on the page

Default: bottom-right

| Value | Description |
|-------|-------------|
| `bottom-right` | Bottom-right corner (default) |
| `bottom-left` | Bottom-left corner |
| `top-right` | Top-right corner |
| `top-left` | Top-left corner |

# `direction`

**Type:** `SiteActionsDirection` (optional)

Direction for action buttons to expand

Default: vertical

| Value | Description |
|-------|-------------|
| `vertical` | Vertical stack (default) - buttons expand upward/downward from corner |
| `horizontal` | Horizontal row - buttons expand left/right from corner |

# `mode`

**Type:** `SiteActionsMode` (optional)

Display mode for the actions zone

Default: collapsed

| Value | Description |
|-------|-------------|
| `collapsed` | Collapsed (default) - main FAB expands on click to reveal action buttons |
| `expanded` | Expanded - all action buttons always visible, no main FAB |


***

This documentation was generated from [`site.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/site.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
