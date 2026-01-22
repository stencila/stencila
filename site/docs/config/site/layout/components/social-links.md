---
title: Site Layout SocialLinks Component
description: Social/external links (GitHub, Discord, LinkedIn, etc.)
---

Social/external links (GitHub, Discord, LinkedIn, etc.)

Displays links to social media and external platforms with automatic icons.
Uses `site.socials` as the primary data source. Component config can filter
the site-level configuration or add custom links.

**Ordering:** Links from `site.socials` appear in the order defined there.
Use `include` to filter and reorder. Custom links are always appended, e.g.

```toml
# Social links plus footer component config
[site.socials]
github = "org/repo"
discord = "invite-code"
x = "handle"

[site.layout.footer]
end = "social-links"  # Uses all site.socials in order defined above

# Filter and reorder with include (discord first, then github, x excluded):
end = { type = "social-links", include = ["discord", "github"] }

# Add custom links (appended after site.socials):
end = { type = "social-links", custom = [{ name = "Blog", url = "https://blog.example.com", icon = "lucide:rss" }] }
```

## `style`

**Type:** `SocialLinksStyle` (optional)

Display style

Default: `icon`

| Value | Description |
|-------|-------------|
| `icon` | Icons only (default) |
| `text` | Text labels only |
| `both` | Icon and text |

## `new-tab`

**Type:** `boolean` (optional)

Whether links open in new tab

When true, links include target="_blank" and rel="noopener noreferrer".

Default: true

## `include`

**Type:** `array` (optional)

Filter to specific platforms and optionally reorder

Only platforms listed here (and present in `site.socials`) will be shown,
in the order specified. Default: all platforms from site.socials in their
defined order.

## `exclude`

**Type:** `array` (optional)

Exclude these platforms (validated against known platforms + "custom")

Exclude takes precedence over include.

## `custom`

**Type:** `array` (optional)

Custom links for platforms not in the known set

Use this for blogs, documentation sites, or platforms without built-in
icon support. Each entry needs a name and URL; icon is optional.
Custom links are always appended after `site.socials` links.


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
