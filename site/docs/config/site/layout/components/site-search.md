---
title: Site Layout SiteSearch Component
description: Site search
---

Site search

Displays a search button that opens a search modal when clicked.
Requires `site.search.enabled = true` in config to generate the search index.

```toml
# Enable search index generation
[site.search]
enabled = true

# Add site-search to header
[site.layout.header]
end = "site-search"

# Or with custom placeholder:
end = { type = "site-search", placeholder = "Search docs..." }
```

## `placeholder`

**Type:** `string` (optional)

Placeholder text for the search input

Default: "Search..."


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
