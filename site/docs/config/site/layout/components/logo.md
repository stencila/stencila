---
title: Site Layout Logo Component
description: Site logo image with responsive and dark mode variants
---

Site logo image with responsive and dark mode variants

When used as a bare `"logo"` string, inherits configuration from
`site.logo`. When used as an object, can override any fields.

```toml
# Header logo using site defaults and overrides
[site.layout.header]
start = "logo"  # Uses site.logo config

# Or with overrides:
start = { type = "logo", default = "header-logo.svg", dark = "header-logo-dark.svg" }
```

## `default`

**Type:** `string` (optional)

Default logo image path (used for desktop light mode)

## `mobile`

**Type:** `string` (optional)

Logo for mobile breakpoint (< 640px)

## `tablet`

**Type:** `string` (optional)

Logo for tablet breakpoint (640px - 768px)

## `dark`

**Type:** `string` (optional)

Logo for dark mode (desktop)

## `dark-mobile`

**Type:** `string` (optional)

Logo for dark mode on mobile

## `dark-tablet`

**Type:** `string` (optional)

Logo for dark mode on tablet

## `link`

**Type:** `string` (optional)

Link target when logo is clicked (default: "/")

## `alt`

**Type:** `string` (optional)

Alt text for accessibility (used as aria-label on the link)


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
