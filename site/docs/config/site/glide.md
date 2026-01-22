---
title: Site Glide Config
description: Glide configuration for client-side navigation
---

Glide configuration for client-side navigation

When enabled, internal link clicks are intercepted and content
is swapped without full page reloads, using View Transitions API
when available, e.g.
```toml
# Prefetch more pages for glide navigation
[site.glide]
prefetch = 25
```

**Type:** `GlideConfig`

# `enabled`

**Type:** `boolean` (optional)

Enable client-side navigation

When true, internal links use AJAX navigation with View Transitions.
Default: true

# `prefetch`

**Type:** `integer` (optional)

Maximum prefetches per session

Pages are fetched on hover/focus before click, up to this limit.
Set to 0 to disable prefetching. Only applies when glide is enabled.
Default: 20

# `cache`

**Type:** `integer` (optional)

Maximum number of pages to cache

Controls how many pages are kept in the LRU cache for instant
back/forward navigation. Set to 0 to disable caching.
Default: 10


***

This documentation was generated from [`site.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/site.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
