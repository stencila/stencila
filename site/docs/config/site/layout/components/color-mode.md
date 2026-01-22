---
title: Site Layout ColorMode Component
description: Light/dark mode toggle
---

Light/dark mode toggle

Toggles between light and dark color modes. The user's preference is
saved to localStorage and persists across sessions. On first visit,
respects the operating system's color scheme preference.

```toml
[site.layout.header]
end = "color-mode"  # Icon-only (default)

# With text label:
end = { type = "color-mode", style = "both" }

# Text only (no icon):
end = { type = "color-mode", style = "label" }
```

## `style`

**Type:** `ColorModeStyle` (optional)

Display style

Default: `icon`

| Value | Description |
|-------|-------------|
| `icon` | Sun/moon icon only (default) |
| `label` | "Light"/"Dark" text label only |
| `both` | Icon and label |


***

This documentation was generated from [`layout/components.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/layout/components.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
