---
title: Site Sitemap Config
description: Sitemap configuration for generated site routes
---

Sitemap configuration for generated site routes

When enabled, sitemap files are generated during site rendering using
the canonical site URL. XML and text sitemap formats are supported.

Can be a simple boolean or a detailed configuration object, e.g.
```toml
# Enable sitemap generation with defaults
[site]
sitemap = true

# Customize sitemap generation
[site.sitemap]
enabled = true
formats = ["xml", "txt"]
visibility = "public-only"
exclude-routes = ["/drafts/**"]
include-lastmod = true
```

**Type:** `SitemapConfig`

# `enabled`

**Type:** `boolean` (optional)

Enable sitemap generation

When true, sitemap files are generated during site rendering.
Default: false

# `formats`

**Type:** `array` (optional)

Sitemap formats to generate

Default: `["xml"]`

# `visibility`

**Type:** `SitemapVisibility` (optional)

Visibility policy for included routes

- `public-only`: only public routes, excludes specimen
- `all`: includes restricted routes and specimen

Default: `public-only`

| Value | Description |
|-------|-------------|
| `public-only` | Include only public routes and exclude the specimen page |
| `all` | Include all routes, including restricted routes and the specimen page |

# `exclude-routes`

**Type:** `array` (optional)

Route patterns to exclude from sitemap generation

Glob patterns for routes that should not be emitted to sitemap files.

Example: `["/drafts/**", "/internal/**"]`

# `include-lastmod`

**Type:** `boolean` (optional)

Include last-modified timestamps where available

When true, source file modification times are included where they can
be determined.
Default: true


***

This documentation was generated from [`site.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/site.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
