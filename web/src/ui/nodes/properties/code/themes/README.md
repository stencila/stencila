# CodeMirror Themes

This directory contains CodeMirror theme modules for Stencila. Each theme is a standalone module that exports a single function returning a CodeMirror Extension.

## Structure

```
themes/
├── index.ts          # Main theme selector
├── custom.ts         # CSS variable-based theme
├── basic-light.ts    # Basic Light theme
├── basic-dark.ts     # Basic Dark theme
└── README.md         # This file
```

## Usage

Themes are selected via the `--code-theme` CSS variable:

```css
:root {
  --code-theme: basic-light;  /* Use a named theme */
}

/* Or for full customization */
:root {
  --code-theme: custom;  /* Use CSS variables */
  --code-comment: #008000;
  --code-keyword: #0000ff;
  /* ... etc */
}
```

## Available Themes

Currently implemented:

**Customizable:**
- `custom` - CSS variable-based theme (default fallback)

**Light themes:**
- `basic-light` - Light theme with good contrast (default)
- `github-light` - GitHub Light theme
- `gruvbox-light` - Gruvbox Light theme with warm retro colors
- `material-light` - Material Design Light theme
- `solarized-light` - Solarized Light theme with precision colors
- `tokyo-night-day` - Tokyo Night Day theme variant
- `vscode-light` - VS Code Light theme

**Dark themes:**
- `abcdef` - Abcdef theme with signature blue colors
- `abyss` - Abyss theme with deep ocean-inspired colors
- `android-studio` - Android Studio/IntelliJ IDEA dark theme
- `andromeda` - Andromeda theme with vibrant purple and teal accents
- `basic-dark` - Basic dark theme
- `forest` - Forest theme with natural green and earth tones
- `github-dark` - GitHub Dark theme
- `gruvbox-dark` - Gruvbox Dark theme with muted warm colors
- `material-dark` - Material Design Dark theme
- `monokai` - Monokai theme with classic colors
- `nord` - Nord theme with arctic, north-bluish color palette
- `palenight` - Palenight theme with soft purple and blue tones
- `solarized-dark` - Solarized Dark theme with precision colors
- `tokyo-night-storm` - Tokyo Night Storm theme variant
- `volcano` - Volcano theme with red and fire-like colors
- `vscode-dark` - VS Code Dark theme

## Adding New Themes

To add a new theme (e.g., "monokai"):

### 1. Create the theme file: `monokai.ts`

```typescript
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { Extension } from '@codemirror/state'
import { tags } from '@lezer/highlight'

/**
 * Monokai Theme for CodeMirror
 */
export function createMonokaiTheme(): Extension {
  // Color palette
  const base00 = '#272822' // Background
  const base01 = '#f8f8f2' // Foreground
  // ... add more colors

  const highlightStyle = HighlightStyle.define([
    { tag: tags.keyword, color: base0A },
    { tag: tags.string, color: base0B },
    // ... add more tag mappings
  ])

  return syntaxHighlighting(highlightStyle)
}
```

### 2. Update `index.ts`

Add the import:
```typescript
import { createMonokaiTheme } from './monokai'
```

Add to the switch statement:
```typescript
case 'monokai':
  return createMonokaiTheme()
  break
```

### 3. Update CSS documentation

In `web/src/themes/base/code.css`, add to the available themes list:

```css
/* Available named themes:
 * Light: basic-light, ...
 * Dark: basic-dark, monokai, ...
 */
```
