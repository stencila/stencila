---
title: Site Remotes Config
description: Site remotes configuration
---

Site remotes configuration

Enables users to add Google Docs or Microsoft 365 documents to the repository
via GitHub PRs, with optional bi-directional sync. Requires `workspace.id`
to be configured for cloud enforcement of public/anon settings.
Position is controlled via `[site.actions]`.

Can be a simple boolean or a detailed configuration object, e.g.
```toml
# Enable remotes with defaults
[site]
remotes = true

# Detailed remotes configuration
[site.remotes]
enabled = true
path = "content"               # Default target directory
default-format = "smd"         # Stencila Markdown
allowed-formats = ["smd", "md"]
default-sync-direction = "bi"  # Bi-directional sync
public = false                 # Only show to authenticated users
anon = false                   # Require GitHub auth
user-path = true               # Allow custom target paths
require-message = false        # Optional PR description
include = ["docs/**"]          # Only show on docs pages
exclude = ["api/**"]           # Hide from API reference
```

**Type:** `SiteRemotesConfig`

# `enabled`

**Type:** `boolean`

Whether remote document adding is enabled

When false, the remote widget is not rendered.

# `public`

**Type:** `boolean` (optional)

Whether public (non-team members) can add remote documents

This is enforced server-side by Stencila Cloud. When false,
the remote widget is hidden from non-authenticated users.
Default: false (more restrictive)

# `anon`

**Type:** `boolean` (optional)

Whether anonymous (no GitHub auth) submissions are allowed

This is enforced server-side by Stencila Cloud. When false,
users must connect their GitHub account to add remote documents.
Default: false

# `path`

**Type:** `string` (optional)

Default target directory for new remote documents

Path is relative to repo root.
Example: "content" or "docs"

# `default-format`

**Type:** `SiteRemoteFormat` (optional)

Default output format for pulled documents

Default: smd (Stencila Markdown)

| Value | Description |
|-------|-------------|
| `smd` | Stencila Markdown (default) |
| `md` | Standard Markdown |
| `html` | HTML |

# `allowed-formats`

**Type:** `array` (optional)

Allowed output formats

If specified, users can only choose from these formats.
Default: all formats allowed

# `default-sync-direction`

**Type:** `SiteRemoteSyncDirection` (optional)

Default sync direction

Default: bi (bi-directional)

| Value | Description |
|-------|-------------|
| `from-remote` | Changes in remote doc create PRs to update repo |
| `bi` | Changes sync both ways (default) |
| `to-remote` | Changes in repo update remote doc |

# `include`

**Type:** `array` (optional)

Glob patterns for paths to show widget on

If specified, widget is only shown on pages matching these patterns.
Example: `["docs/**", "content/**"]`

# `exclude`

**Type:** `array` (optional)

Glob patterns for paths to hide widget from

Widget is hidden on pages matching these patterns.
Example: `["api/**", "internal/**"]`

# `spread-routes`

**Type:** `boolean` (optional)

Show remote widget on spread routes (virtual routes from templates)

When true, remotes are shown on spread routes like `/{region}/`.
When false (default), remotes are hidden on spread routes to avoid
confusion about where documents are added (documents go to a fixed
directory, not the virtual route path).


***

This documentation was generated from [`site.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/site.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
