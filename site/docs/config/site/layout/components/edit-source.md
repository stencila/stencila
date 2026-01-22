---
title: Site Layout EditSource Component
description: Edit source link for GitHub/GitLab/Bitbucket
---

Edit source link for GitHub/GitLab/Bitbucket

Displays a link to edit the current page on the source repository.
Auto-detects the repository from git origin for github.com, gitlab.com,
and bitbucket.org. For self-hosted instances or other platforms, use
the `base-url` option.

The icon shows the platform logo (GitHub, GitLab, or Bitbucket), the
default text is "Edit on <Platform>", and hovering shows "Edit source on <Platform>", e.g.

```toml
# Edit-source links with defaults and overrides
[site.layout.footer]
end = "edit-source"  # Auto-detect from git origin

# With custom text:
end = { type = "edit-source", text = "Suggest changes" }

# For self-hosted GitLab:
end = { type = "edit-source", base-url = "https://gitlab.mycompany.com/team/docs/-/edit/main/" }
```

## `text`

**Type:** `string` (optional)

Custom link text

Default: "Edit on <Platform>" or "Edit source" for custom base-url

## `style`

**Type:** `EditSourceStyle` (optional)

Display style

Default: both

| Value | Description |
|-------|-------------|
| `icon` | Pencil/edit icon only |
| `text` | Text only |
| `both` | Icon and text (default) |

## `base-url`

**Type:** `string` (optional)

Full edit URL prefix (e.g., "https://github.com/org/repo/edit/main/")

When provided, the file path is simply appended. Required for
self-hosted instances or unsupported platforms (Gitea, Forgejo, etc).

## `branch`

**Type:** `string` (optional)

Override branch name for auto-detected URLs

Ignored when `base-url` is provided.

Default: auto-detect or "main"

## `path-prefix`

**Type:** `string` (optional)

Path prefix within repo (e.g., "docs/" if content is in a subdirectory)


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
