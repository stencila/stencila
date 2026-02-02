---
title: Site Search Config
description: Search configuration for client-side full-text search
---

Search configuration for client-side full-text search

When enabled, a search index is generated during site rendering.
The index is sharded for efficient incremental loading and supports
Unicode text with diacritic folding.

Can be a simple boolean or a detailed configuration object, e.g.
```toml
# Enable search with defaults
[site]
search = true

# Customize search indexing
[site.search]
enabled = true
include-types = ["Heading", "Paragraph"]
exclude-routes = ["/api/**"]
```

**Type:** `SearchConfig`

# `enabled`

**Type:** `boolean` (optional)

Enable search index generation

When true, a search index is generated during site rendering.
Default: false

# `include-types`

**Type:** `array` (optional)

Node types to include in the search index

Specifies which Stencila node types should be indexed.
When not specified, defaults to common content types:
`["Heading", "Paragraph", "Datatable", "CodeChunk", "Figure", "Table"]`

Available types include: Article, Heading, Paragraph, Datatable,
CodeChunk, Figure, Table, and other Stencila node types.

# `exclude-routes`

**Type:** `array` (optional)

Route patterns to exclude from indexing

Glob patterns for routes that should not be indexed.
Useful for excluding API documentation, internal pages, etc.

Example: `["/api/**", "/internal/**"]`

# `max-text-length`

**Type:** `integer` (optional)

Maximum text length per search entry

Text content longer than this will be truncated.
Default: 500 characters

# `fuzzy`

**Type:** `boolean` (optional)

Enable fuzzy search support

When true, pre-computed trigrams are included in the search index
to enable fuzzy matching (finding results with typos).
This increases index size by approximately 1KB per entry.
Default: true


***

This documentation was generated from [`site.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/site.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
