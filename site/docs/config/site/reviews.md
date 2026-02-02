---
title: Site Reviews Config
description: Site reviews configuration
---

Site reviews configuration

Enables readers to submit comments and suggestions on site pages.
Requires `workspace.id` to be configured for cloud enforcement of
public/anon settings. Position is controlled via `[site.actions]`.

Can be a simple boolean or a detailed configuration object, e.g.
```toml
# Enable reviews with defaults
[site]
reviews = true

# Detailed reviews configuration
[site.reviews]
enabled = true
public = true           # Non-team members can submit
anon = false            # Require GitHub auth
types = ["comment", "suggestion"]
min-selection = 10
max-selection = 5000
shortcuts = false
include = ["docs/**"]   # Only show on docs pages
exclude = ["api/**"]    # Hide from API reference
```

**Type:** `SiteReviewsConfig`

# `enabled`

**Type:** `boolean`

Whether reviews are enabled

When false, the review widget is not rendered.

# `public`

**Type:** `boolean` (optional)

Whether public (non-team members) can submit reviews

This is enforced server-side by Stencila Cloud. When false,
the review widget is hidden from non-authenticated users.
Default: true

# `anon`

**Type:** `boolean` (optional)

Whether anonymous (no GitHub auth) submissions are allowed

This is enforced server-side by Stencila Cloud. When false,
users must connect their GitHub account to submit reviews.
Default: false

# `types`

**Type:** `array` (optional)

Allowed review item types

Default: both comment and suggestion

# `min-selection`

**Type:** `integer` (optional)

Minimum characters required to trigger the widget

Prevents accidental tiny selections from showing the review buttons.
Default: 1

# `max-selection`

**Type:** `integer` (optional)

Maximum characters allowed in a selection

Prevents selecting excessively large amounts of text.
Default: 5000

# `shortcuts`

**Type:** `boolean` (optional)

Enable keyboard shortcuts for reviews

When enabled:
- Ctrl+Shift+C: Add comment on current selection
- Ctrl+Shift+S: Add suggestion on current selection
- Escape: Cancel current input / clear selection

Default: true

# `include`

**Type:** `array` (optional)

Glob patterns for paths to show reviews on

If specified, reviews are only shown on pages matching these patterns.
Example: `["docs/**", "guides/**"]`

# `exclude`

**Type:** `array` (optional)

Glob patterns for paths to hide reviews from

Reviews are hidden on pages matching these patterns.
Example: `["api/**", "changelog/**"]`

# `spread-routes`

**Type:** `boolean` (optional)

Show review widget on spread routes (virtual routes from templates)

When true, reviews are shown on spread routes like `/{region}/`.
When false (default), reviews are hidden on spread routes to avoid
confusion about which document is being reviewed.


***

This documentation was generated from [`reviews.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/reviews.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
