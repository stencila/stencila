---
title: Site Configuration
description: Configuration for a site
---

# Site Configuration

Configuration for a site

Site settings are associated with a workspace (see `WorkspaceConfig`).
The workspace ID is used to identify the site in Stencila Cloud.

Example:
```toml
[site]
domain = "docs.example.org"
root = "docs"
exclude = ["**/*.draft.md", "_drafts/**"]

[site.routes]
"/" = "index.md"
"/about/" = "README.md"
```

## Properties

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

### `routes`

**Type:** `HashMap` (optional)

Custom routes for serving content

Routes map URL paths to files, redirects, or spread configurations.
The key is the URL path (or path template for spreads), and the value can be:
- A simple string for the file path: `"/about/" = "README.md"`
- An object for redirects: `"/old/" = { redirect = "/new/", status = 301 }`
- An object for spreads: `"/{region}/" = { file = "report.smd", arguments = { region = ["north", "south"] } }`

Example:
```toml
[site.routes]
"/" = "index.md"
"/about/" = "README.md"
"/old-page/" = { redirect = "/new-page/", status = 301 }
"/{region}/{species}/" = { file = "report.smd", arguments = { region = ["north", "south"], species = ["ABC", "DEF"] } }
```

