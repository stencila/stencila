---
title: Site Uploads Config
description: Site uploads configuration
---

Site uploads configuration

Enables users to upload files (e.g., CSV data updates) to the repository
via GitHub PRs. Requires `workspace.id` to be configured for cloud
enforcement of public/anon settings. Position is controlled via `[site.actions]`.

Can be a simple boolean or a detailed configuration object, e.g.
```toml
# Enable uploads with defaults
[site]
uploads = true

# Detailed uploads configuration
[site.uploads]
enabled = true
path = "data"           # Widget on /data/**, files to data/
allowed-types = ["csv", "json", "xlsx"]
max-size = 10485760     # 10MB
public = false          # Only show to authenticated users
anon = false            # Require GitHub auth
user-path = true        # Allow custom paths
allow-overwrite = true  # Can replace existing files
require-message = true  # Must provide description
```

**Type:** `SiteUploadsConfig`

# `enabled`

**Type:** `boolean`

Whether uploads are enabled

When false, the upload widget is not rendered.

# `public`

**Type:** `boolean` (optional)

Whether public (non-team members) can upload files

This is enforced server-side by Stencila Cloud. When false,
the upload widget is hidden from non-authenticated users.
Default: false

# `anon`

**Type:** `boolean` (optional)

Whether anonymous (no GitHub auth) submissions are allowed

This is enforced server-side by Stencila Cloud. When false,
users must connect their GitHub account to upload files.
Default: false

# `path`

**Type:** `string` (optional)

Default target directory for uploaded files

Path is relative to repo root.
Example: "data" or "uploads"

# `include`

**Type:** `array` (optional)

Override: glob patterns for pages to show widget on

If specified, overrides the visibility derived from `path`.
Example: `["admin/**", "dashboard/**"]`

# `exclude`

**Type:** `array` (optional)

Glob patterns for pages to hide widget from

Widget is hidden on pages matching these patterns.
Example: `["api/**", "internal/**"]`

# `extensions`

**Type:** `array` (optional)

File extensions to include in the `_files` index

When specified, only files with these extensions are indexed.
When `None` (default), all files are indexed.
Extensions are matched case-insensitively, without leading dot.
Example: `["csv", "json", "xlsx"]`

# `spread-routes`

**Type:** `boolean` (optional)

Show upload widget on spread routes (virtual routes from templates)

When true, uploads are shown on spread routes like `/{region}/`.
When false (default), uploads are hidden on spread routes to avoid
confusion about where files are uploaded (files go to the source
file's directory, not the virtual route path).


***

This documentation was generated from [`site.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/site.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
