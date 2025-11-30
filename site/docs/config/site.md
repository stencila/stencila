---
title: Site Configuration
description: Configuration for a site
---

# Site Configuration

Configuration for a site

Example:
```toml
[site]
id = "s123456789"
watch = "wAbCdEfGh1"
domain = "docs.example.org"
root = "docs"
exclude = ["**/*.draft.md", "_drafts/**"]
```

## Properties

### `id`

**Type:** `string` (optional)
**Pattern:** `^s[a-z0-9]{9}$`

The id of the Stencila Site

Returned by Stencila Cloud when a site is created.

### `watch`

**Type:** `string` (optional)
**Pattern:** `^w[a-zA-Z0-9]{9}$`

Watch ID from Stencila Cloud

If watching is enabled for this site, this field contains the watch ID.
The watch enables unidirectional sync from repository to site - when
changes are pushed to the repository, the site is automatically updated.

### `domain`

**Type:** `string` (optional)
**Pattern:** `^([a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,}$`

Custom domain for the site

This is a cached value that is kept in sync with Stencila Cloud
when site details are fetched or the domain is modified.
The canonical source is the Stencila Cloud API.

### `root`

**Type:** `string (path)` (optional)

Root directory for site content

Path relative to the config file containing this setting.
When set, only files within this directory will be published
to the site, and routes will be calculated relative to this
directory rather than the workspace root.

Example: If set to "docs" in /myproject/stencila.toml,
then /myproject/docs/guide.md → /guide/ (not /docs/guide/)

### `exclude`

**Type:** `Vec` (optional)

Glob patterns for files to exclude when publishing

Files matching these patterns will be excluded from publishing.
Exclude patterns take precedence over include patterns.
Patterns are relative to `root` (if set) or the workspace root.
Default exclusions (`.git/`, `node_modules/`, etc.) are applied automatically.

Example: `["**/*.draft.md", "temp/**"]`

