---
title: Site Layout Copyright Component
description: Copyright notice with auto-updating year
---

Copyright notice with auto-updating year

Displays a copyright notice with optional auto-updating year.
When used as a bare `"copyright"` string, uses `site.author` as the holder
and current year, e.g.

```toml
# Footer copyright variants
[site.layout.footer]
middle = "copyright"  # Uses site.author, current year

# With year range:
middle = { type = "copyright", start-year = 2020 }

# With custom holder:
middle = { type = "copyright", holder = "Acme Inc", link = "https://acme.com" }

# Full custom text (no auto-year):
middle = { type = "copyright", text = "Custom copyright notice" }
```

## `text`

**Type:** `string` (optional)

Full custom text (overrides all other fields)

When provided, this text is used verbatim with no auto-year.
Example: "© 2024 Acme Inc. All rights reserved."

## `holder`

**Type:** `string` (optional)

Copyright holder name (defaults to site.author)

Example: "Acme Inc"

## `start-year`

**Type:** `integer` (optional)

Start year for copyright range (e.g., 2020 in "2020-2024")

If not set, only current year is shown.

## `link`

**Type:** `string` (optional)

Link URL for the holder name

When provided, the holder name becomes a clickable link.


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
