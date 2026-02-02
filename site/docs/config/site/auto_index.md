---
title: Site Auto_index Config
description: Auto-index configuration for directories without content files
---

Auto-index configuration for directories without content files

When enabled (default), directories that appear in navigation but lack
content files (main.md, README.md, index.md, etc.) will have index pages
auto-generated listing their child pages with links.

Can be a simple boolean or a detailed configuration object, e.g.
```toml
# Enable auto-index (default)
[site]
auto-index = true

# Disable auto-index
[site]
auto-index = false

# Enable with exclusions
[site.auto-index]
enabled = true
exclude = ["/api/**", "/internal/**"]
```

**Type:** `AutoIndexConfig`

# `enabled`

**Type:** `boolean` (optional)

Enable auto-index generation

When true, directories without index files get auto-generated pages.
Default: true

# `exclude`

**Type:** `array` (optional)

Glob patterns for routes to exclude from auto-indexing

Routes matching any of these patterns will not have auto-generated
index pages, even if they lack content files.

Example: `["/api/**", "/internal/**"]`


***

This documentation was generated from [`site.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/site.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
