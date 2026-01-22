---
title: Site Layout Logo Component
description: Site logo image with responsive and dark mode variants
---

Site logo image with responsive and dark mode variants

When used as a bare `"logo"` string, inherits configuration from
`site.logo`. When used as an object, can override any fields.

Available fields (all optional, inherit from `site.logo` if not specified):
- `default`: Default logo image path (desktop light mode)
- `dark`: Logo for dark mode (desktop)
- `mobile`: Logo for mobile breakpoint (< 640px)
- `tablet`: Logo for tablet breakpoint (640px - 768px)
- `dark-mobile`: Logo for dark mode on mobile
- `dark-tablet`: Logo for dark mode on tablet
- `link`: Link target when clicked (default: "/")
- `alt`: Alt text for accessibility

```toml
# Header logo using site defaults and overrides
[site.layout.header]
start = "logo"  # Uses site.logo config

# Or with overrides:
start = { type = "logo", default = "header-logo.svg", dark = "header-logo-dark.svg" }
```


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
