---
title: Site Layout PrevNext Component
description: Previous/next page navigation links
---

Previous/next page navigation links

Displays links to previous and next pages in the navigation sequence.
Supports keyboard shortcuts: `j` or `←` for previous, `k` or `→` for next.
The sequence follows `site.nav` order if configured.

```toml
[site.layout.bottom]
middle = "prev-next"  # Standard style (default)

# Compact style (icons + labels only):
middle = { type = "prev-next", style = "compact" }

# Custom labels for localization:
middle = { type = "prev-next", prev-text = "Précédent", next-text = "Suivant" }
```

## `style`

**Type:** `PrevNextStyle` (optional)

Display style

Default: standard

| Value | Description |
|-------|-------------|
| `minimal` | Minimal: just arrow icons |
| `compact` | Compact: icons + labels |
| `standard` | Standard: icons + labels + page titles (default) |
| `detailed` | Detailed: icons + labels + titles + position indicator |

## `prev-text`

**Type:** `string` (optional)

Custom text for previous link

Useful for localization.

Default: "Previous"

## `next-text`

**Type:** `string` (optional)

Custom text for next link

Useful for localization.

Default: "Next"

## `separator`

**Type:** `string` (optional)

Separator between prev and next links

Common values: "|", "·", or any custom string.
Only shown when both prev and next links are present.

Default: none


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
