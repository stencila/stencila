---
title: Themes
description: |
  Overview of the Stencila theme system and how to create custom themes.
---

# Overview

Stencila themes are a design-token system plus optional CSS rules that together define the visual identity of a document. Tokens are expressed as CSS custom properties (design tokens) so typography, spacing, color, and layout decisions stay consistent across screen and print. Those same tokens are also consumed by non-web renderers (e.g. plotting backends, DOCX, email, PDF) so a single theme can be translated into multiple output formats.

Semantic tokens act as the stable public API for theme authors, while node-specific modules expose focused component tokens. This keeps theme customization predictable, minimizes duplicated overrides, and lets responsive/print adjustments cascade automatically.

- **Design tokens (CSS variables)** for fonts, colors, spacing, layout, and component styling
- **CSS rules** for the base document and component modules (e.g. headings, tables, citations)
- **Translation hooks** that map tokens to non-web outputs (plots, Word, email, PDF)

# Resolution and Computation

Theme resolution and computation are part of the implementation contract that affects which tokens are available to non-web targets:

- **Resolution order** (when no theme is specified): workspace `theme.css` (walks up from the document path), then user `default.css`, then builtin `stencila.css`.
- **Named themes**: if you request a theme by name, Stencila searches user themes first, then builtin themes.
- **Plot presets**: if `--plot-theme` is set to a named preset (not `custom` or `none`), preset CSS from `themes/plots/{name}.css` is merged into the theme before variables are extracted.
- **Token extraction scope**: only top-level `:root` custom properties are parsed for token export. Tokens inside `@media`/`@supports` blocks are used for web rendering but are not exported to non-web targets.
- **Evaluation and conversion**: `var()` references and `calc()`/`color-mix()` expressions are resolved, colors are normalized to hex, and lengths are converted to the target unit system (points, pixels, inches, or twips).
- **Runtime overrides**: computed variables can be merged with document metadata overrides (e.g. `--document-*`) for features like margin box content.
- **Caching and watching**: theme resolution is cached for ~30 seconds in the runtime; tooling that needs immediate updates uses the watch API to reload on file changes.

# Usage

Theme CSS can include both tokens and rules; tokens are the preferred way to express design decisions that need to carry across outputs.

Theme authors typically start by adjusting a small set of semantic tokens:

- `--text-font-family`, `--heading-font-family`, and `--code-font-family` for typography
- `--text-color-primary`, `--color-accent`, and `--surface-background` for color and contrast
- `--content-spacing` and `--content-width` for layout rhythm
- `--border-radius-default` and `--border-color-default` for component framing

Primitive tokens live in `tokens-primitive.css`, while the public, stable API is in `tokens-semantic.css`. Override semantic tokens first, then reach for module tokens when you need component-specific changes.

# Token Types

These reference pages provide token lists and examples by Stencila node type:

- [Admonitions](tokens/nodes/admonitions.smd)
- [Articles](tokens/nodes/articles.smd)
- [Breaks](tokens/nodes/breaks.smd)
- [Citations](tokens/nodes/citations.smd)
- [Code](tokens/nodes/code.smd)
- [Datatables](tokens/nodes/datatables.smd)
- [Diagrams](tokens/nodes/diagrams.smd)
- [Figures](tokens/nodes/figures.smd)
- [Headings](tokens/nodes/headings.smd)
- [Images](tokens/nodes/images.smd)
- [Links](tokens/nodes/links.smd)
- [Lists](tokens/nodes/lists.smd)
- [Math](tokens/nodes/math.smd)
- [Paragraphs](tokens/nodes/paragraphs.smd)
- [Plots](tokens/nodes/plots.smd)
- [Quotes](tokens/nodes/quotes.smd)
- [References](tokens/nodes/references.smd)
- [Tables](tokens/nodes/tables.smd)

Site layout and navigation components use a separate set of tokens:

- [Site Layout](tokens/site/layout.smd)
- [Navigation Menu](tokens/site/nav-menu.smd)
- [Navigation Tree](tokens/site/nav-tree.smd)
- [Navigation Groups](tokens/site/nav-groups.smd)
- [Breadcrumbs](tokens/site/breadcrumbs.smd)
- [Table of Contents Tree](tokens/site/toc-tree.smd)
- [Prev/Next Navigation](tokens/site/prev-next.smd)
- [Edit On](tokens/site/edit-on.smd)
- [Edit Source](tokens/site/edit-source.smd)
- [Copy Markdown](tokens/site/copy-markdown.smd)
- [Social Links](tokens/site/social-links.smd)
- [Logo](tokens/site/logo.smd)
- [Site Title](tokens/site/title.smd)
- [Copyright](tokens/site/copyright.smd)
- [Site Review](tokens/site/site-review.smd)

For general font and prints layout tokens see:

- [Fonts](tokens/nodes/fonts.smd)
- [Print](tokens/nodes/print.smd)

# Targets

Because tokens are CSS variables, Stencila can translate the same theme into non-web environments. This lets you define a theme once and deploy it across multiple media. Each target page documents how tokens are mapped for that output:

| Target       | What gets themed                           | Reference                   |
| ------------ | ------------------------------------------ | --------------------------- |
| Word (DOCX)  | Styles, headings, tables, page layout      | [Word](targets/word.md)     |
| Email        | MJML/HTML typography and component styles  | [Email](targets/email.md)   |
| PDF          | Paged media size, margins, headers/footers | [PDF](targets/pdf.md)       |
| R plots      | Base graphics + ggplot2 defaults           | [R](targets/r.md)           |
| Python plots | Matplotlib rcParams                        | [Python](targets/python.md) |
| Web plots    | JS renderers (Plotly, Vega-Lite, ECharts)  | [Web/JS](targets/web.md)    |

# Architecture

The theme system is built with a modular architecture that allows you to choose exactly what you need:

## Base Theme Entry Point (`base/index.css`)

The base theme entry point imports a collection of focused, self-contained modules for screen rendering:

### Token Layers and Root Styles

- **`tokens-primitive.css`** - Primitive design tokens (raw colors, spacing, font stacks)
- **`tokens-semantic.css`** - Semantic design tokens with dark/print variants
- **`root.css`** - Base document styles and global resets
- **`browsers.css`** - Browser normalization and cross-browser adjustments
- **`pages.css`** - Paged media (`@page`) tokens and margin box rules

### Typography Modules

- **`links.css`** - Link reset, styling, focus states, and accessibility
- **`headings.css`** - Complete heading system with reset, h1-h6 styles, and mobile adjustments
- **`paragraphs.css`** - Paragraph styling including lead paragraphs and mobile optimizations
- **`lists.css`** - List reset, ul/ol/li styling, nesting, and mobile adjustments

### Content Modules

- **`code.css`** - Inline and block code styling with mobile adjustments
- **`tables.css`** - Table styling, captions, notes, and print variants
- **`datatables.css`** - Datatable styling for typed column tables
- **`figures.css`** - Figure and figcaption styling for Stencila figure components
- **`images.css`** - Image object styling for inline and block images
- **`quotes.css`** - Blockquote styling with mobile adjustments
- **`math.css`** - Mathematical content font styling
- **`breaks.css`** - Thematic break (horizontal rule) styling

### Stencila Components

- **`admonitions.css`** - Stencila admonition component styling
- **`citations.css`** - Citation and citation group styling
- **`diagrams.css`** - Diagram theming (applied via JavaScript integration)
- **`plots.css`** - Plot theming tokens for supported plotting libraries
- **`references.css`** - Bibliographic reference styling

## Print Support

Print and PDF output are handled through `pages.css` (paged media tokens and margin boxes) and print variants embedded in component modules (e.g., tables, figures, code). There is no separate print entry point; print behavior is applied via `@media print` inside each module.

# Import Patterns

## Monolithic

For themes that want everything included automatically:

```css
@import url("./base.css");

:root {
  --text-font-family: "Your Font", serif;
}
```

The `base.css` file imports `index.css` and includes print support via module-level `@media print` rules.

## Selective Imports (Advanced)

For advanced users who need fine-grained control, individual modules can be imported selectively:

```css
/* Import only what you need */
@import url("./base/tokens-primitive.css");
@import url("./base/tokens-semantic.css");
@import url("./base/root.css");
@import url("./base/browsers.css");
@import url("./base/pages.css");
@import url("./base/headings.css");
@import url("./base/paragraphs.css");

:root {
  --text-font-family: "Your Font", serif;
}
```

This approach allows you to:

- Minimize CSS payload for specific use cases
- Override individual modules with custom implementations
- Build specialized themes that only style certain elements
- Debug styling issues by isolating specific modules

**Note**: Each module is self-contained with its own reset, styling, and mobile adjustments co-located for maintainability.
