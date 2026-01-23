---
title: Site Layout NavTree Component
description: Hierarchical navigation tree
---

Hierarchical navigation tree

Displays site navigation from `site.nav` configuration (or auto-generated
from routes if not specified). Supports collapsible groups, active page
highlighting, and keyboard navigation, e.g.

```toml
# Sidebar nav tree with defaults and overrides
[site.layout.left-sidebar]
start = "nav-tree"  # Uses defaults

# Or with configuration:
start = { type = "nav-tree", title = "Documentation", expand-depth = 3, expand-current = true }
```

## `title`

**Type:** `string` (optional)

Optional title above the nav tree (e.g., "Navigation", "Docs")

## `depth`

**Type:** `integer` (optional)

Maximum depth to display

Limits how deep the navigation tree renders. Useful for large sites
where you want to show only top-level sections.

Default: unlimited

## `collapsible`

**Type:** `boolean` (optional)

Whether groups are collapsible

When true, group headers can be clicked to expand/collapse children.
When false, all groups are always expanded.

Default: `true`

## `expand-depth`

**Type:** `integer` (optional)

How deep to expand groups by default

Controls the initial expansion depth for collapsible groups.
- `0` = all groups collapsed
- `1` = only top-level groups expanded
- `2` = groups expanded up to level 2 (default)
- `3` = groups expanded up to level 3

Default: `2`

## `expand-current`

**Type:** `boolean` (optional)

Whether to expand groups containing the current page

When true, groups that are ancestors of the current page are
expanded regardless of `expand-depth`. This keeps navigation
focused on the user's current location.

Default: `true`

## `scroll-to-active`

**Type:** `boolean` (optional)

Auto-scroll nav container to show active item on page load

Default: `true`

## `include`

**Type:** `array` (optional)

Include only items matching these patterns

Supports routes ("/docs/*"), IDs ("#features"), and labels ("Features").
See filtering documentation for pattern syntax.

## `exclude`

**Type:** `array` (optional)

Exclude items matching these patterns

Supports routes ("/docs/*"), IDs ("#features"), and labels ("Features").
Exclude takes precedence over include.

## `icons`

**Type:** `NavTreeIcons` (optional)

Whether to show icons from site.icons

Default: hide

| Value | Description |
|-------|-------------|
| `show` | Show icons from site.icons |
| `hide` | Hide icons (default for nav-tree) |


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
