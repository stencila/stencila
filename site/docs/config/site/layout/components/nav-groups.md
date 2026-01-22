---
title: Site Layout NavGroups Component
description: Footer-style grouped navigation
---

Footer-style grouped navigation

Displays flat navigation links organized under headings (e.g., "Products",
"Company", "Resources" sections). Top-level nav items become group headings,
their children become links. Uses CSS grid for responsive auto-columns, e.g.

```toml
# Footer nav groups with filtering
[site.layout.footer]
middle = "nav-groups"  # Uses defaults

# With configuration:
middle = { type = "nav-groups", depth = 2, icons = "hide" }

# Filter specific groups:
middle = { type = "nav-groups", include = ["Products", "Company"] }
```

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

## `depth`

**Type:** `integer` (optional)

Maximum depth to display

Level 1 = group headings, Level 2 = links under headings.
Set to 1 to show only group headings as links.

Default: 2

## `icons`

**Type:** `NavGroupsIcons` (optional)

Whether to show icons on links

Default: `hide`

| Value | Description |
|-------|-------------|
| `show` | Show icons from site.icons |
| `hide` | Hide icons (default for nav-groups) |


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
