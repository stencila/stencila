---
title: Site Logo Config
description: Site logo configuration
---

Site logo configuration

Can be a simple path string or a responsive configuration with
breakpoint and dark mode variants, e.g.
```toml
# Simple logo path
[site]
logo = "logo.svg"

# Responsive logo variants
[site.logo]
default = "logo.svg"
dark = "logo-dark.svg"
mobile = "logo-mobile.svg"
```

**Type:** `LogoConfig`

# `default`

**Type:** `string` (optional)

Default logo image path (used for desktop light mode)

# `mobile`

**Type:** `string` (optional)

Logo for mobile breakpoint (< 640px)

# `tablet`

**Type:** `string` (optional)

Logo for tablet breakpoint (640px - 768px)

# `dark`

**Type:** `string` (optional)

Logo for dark mode (desktop)

# `dark-mobile`

**Type:** `string` (optional)

Logo for dark mode on mobile

# `dark-tablet`

**Type:** `string` (optional)

Logo for dark mode on tablet

# `link`

**Type:** `string` (optional)

Link target when logo is clicked (default: "/")

# `alt`

**Type:** `string` (optional)

Alt text for accessibility (used as aria-label on the link)


***

This documentation was generated from [`site.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/site.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
