---
title: "`stencila snap`"
description: Capture screenshots and measurements of documents served by Stencila
---

Capture screenshots and measurements of documents served by Stencila

The `snap` command allows programmatic screenshotting and measurement of pages served by Stencila. It can be used to:

- Iterate on themes and styled elements and verify changes - Capture screenshots for documentation or CI - Assert computed CSS properties and layout metrics - Measure page elements for automated testing - Extract resolved CSS custom property (theme token) values - Extract the page's color palette - Compare across device viewports using --device

# Usage

```sh
stencila snap [OPTIONS] [ROUTE_OR_PATH]
```

# Examples

```bash
# Snap site root (default route /)
stencila snap --shot homepage.png

# Extract resolved theme token values
stencila snap --tokens

# Extract color palette
stencila snap --palette

# Snap a specific site route with site chrome measurements
stencila snap /docs/guide/ --measure site

# Measure theme-critical regions with grouped color tokens
stencila snap --measure theme --tokens --token-prefix color

# Snap a document file directly
stencila snap ./my-doc.md --shot doc.png

# Assert site chrome properties
stencila snap --assert "exists(stencila-logo)==true"

# Full page dark mode screenshot
stencila snap --dark --full --shot dark-full.png

# Combined: tokens + palette + measurements for theme review
stencila snap --tokens --palette --measure all

# Verify theme token on header
stencila snap --assert "css(stencila-layout > header).backgroundColorHex==#1a1a2e"

# Capture mobile viewport of specific element
stencila snap --device mobile --selector "stencila-article [slot=title]" --shot mobile.png

# Optimize screenshot size for lower image payload cost
stencila snap --shot page.png --resize optimize --max-image-dimension 4096
```

# Arguments

| Name              | Description            |
| ----------------- | ---------------------- |
| `[ROUTE_OR_PATH]` | Route or path to snap. |

# Options

| Name                    | Description                                                                                                         |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `--shot`                | Screenshot output path (.png).                                                                                      |
| `--selector`            | CSS selector to capture or measure.                                                                                 |
| `--full`                | Capture full scrollable page. Possible values: `true`, `false`.                                                     |
| `--device`              | Device preset.                                                                                                      |
| `--width`               | Viewport width in pixels.                                                                                           |
| `--height`              | Viewport height in pixels.                                                                                          |
| `--dpr`                 | Device pixel ratio.                                                                                                 |
| `--resize`              | Screenshot resize mode: never, auto, optimize. Possible values: `never`, `auto`, `optimize`. Default value: `auto`. |
| `--max-image-dimension` | Maximum screenshot dimension in pixels after resize.                                                                |
| `--light`               | Use light color scheme. Possible values: `true`, `false`.                                                           |
| `--dark`                | Use dark color scheme. Possible values: `true`, `false`.                                                            |
| `--print`               | Preview with print media styles (A4 width, for PDF preview). Possible values: `true`, `false`.                      |
| `--wait-until`          | When to capture: load, domcontentloaded, networkidle. Default value: `network-idle`.                                |
| `--wait-for`            | Wait for CSS selector to exist before capturing.                                                                    |
| `--delay`               | Additional delay in milliseconds after page is ready.                                                               |
| `--measure`             | Collect computed CSS and layout metrics.                                                                            |
| `--tokens`              | Extract resolved CSS custom property (theme token) values. Possible values: `true`, `false`.                        |
| `--token-prefix`        | Filter extracted tokens by CSS custom property prefix.                                                              |
| `--palette`             | Extract the page's color palette. Possible values: `true`, `false`.                                                 |
| `--assert`              | Assert measurement conditions.                                                                                      |
| `--url`                 | Override URL (instead of discovering server).                                                                       |

**Possible values of `--measure`**

| Value      | Description                                                      |
| ---------- | ---------------------------------------------------------------- |
| `auto`     | Auto-select based on target type (route → site, path → document) |
| `document` | Document content selectors                                       |
| `site`     | Site chrome selectors                                            |
| `all`      | Both document and site selectors                                 |
| `header`   | Header and top-bar selectors                                     |
| `nav`      | Navigation and breadcrumb selectors                              |
| `main`     | Main content selectors                                           |
| `footer`   | Footer selectors                                                 |
| `theme`    | Combined theme review selectors                                  |

**Possible values of `--device`**

| Value              | Description                         |
| ------------------ | ----------------------------------- |
| `laptop`           | Laptop (1440x900 @2x DPR)           |
| `desktop`          | Desktop (1920x1080 @1x DPR)         |
| `mobile`           | Mobile (390x844 @3x DPR)            |
| `tablet`           | Tablet (768x1024 @2x DPR)           |
| `tablet-landscape` | Tablet Landscape (1024x768 @2x DPR) |

**Possible values of `--wait-until`**

| Value                | Description                       |
| -------------------- | --------------------------------- |
| `load`               | Wait for 'load' event             |
| `dom-content-loaded` | Wait for 'DOMContentLoaded' event |
| `network-idle`       | Wait for network idle (default)   |
