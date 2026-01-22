---
title: Site Layout NavMenu Component
description: Top-level navigation menu bar
---

Top-level navigation menu bar

Displays horizontal navigation with mega-dropdown panels on desktop
and accordion-style menu on mobile. Uses site.nav as data source, e.g.

```toml
# Header nav menu with defaults and overrides
[site.layout.header]
middle = "nav-menu"  # Uses defaults

# Or with configuration:
middle = { type = "nav-menu", groups = "dropdowns", trigger = "click" }
```

## `include`

**Type:** `array` (optional)

Include only items matching these patterns

Supports routes ("/docs/*"), IDs ("#features"), and labels ("Features").

## `exclude`

**Type:** `array` (optional)

Exclude items matching these patterns

Supports routes ("/docs/*"), IDs ("#features"), and labels ("Features").
Exclude takes precedence over include.

## `depth`

**Type:** `integer` (optional)

Maximum depth to display (1 = top-level only)

Default: unlimited

## `groups`

**Type:** `NavMenuGroups` (optional)

How to render groups

Default: `auto`

| Value | Description |
|-------|-------------|
| `auto` | Groups with children become dropdowns, others are links (default) |
| `dropdowns` | All groups become dropdown menus |
| `links` | All groups render as simple links (requires route) |

## `icons`

**Type:** `NavMenuIcons` (optional)

Whether to show icons

Default: `show`

| Value | Description |
|-------|-------------|
| `show` | Show icons on all items that have them (default) |
| `hide` | Never show icons |
| `dropdowns` | Only show icons inside dropdown panels |

## `descriptions`

**Type:** `boolean` (optional)

Whether to show descriptions in dropdowns

Default: `true`

## `trigger`

**Type:** `NavMenuTrigger` (optional)

Dropdown trigger behavior

Default: `hover`

| Value | Description |
|-------|-------------|
| `hover` | Open dropdowns on hover with delay (default) |
| `click` | Open dropdowns on click only |

## `dropdown-style`

**Type:** `NavMenuDropdownStyle` (optional)

Dropdown panel style

Default: `full-width`

| Value | Description |
|-------|-------------|
| `full-width` | Full-width dropdown panels (default) |
| `aligned` | Dropdown aligned to trigger position |


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
