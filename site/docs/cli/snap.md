---
title: "`stencila snap`"
description: Capture screenshots and measurements of documents served by Stencila
---

Capture screenshots and measurements of documents served by Stencila

The `snap` command allows programmatic screenshotting and measurement of documents served by Stencila. It can be used to:

- Iterate on themes and styled elements and verify changes - Capture screenshots for documentation or CI - Assert computed CSS properties and layout metrics - Measure page elements for automated testing

# Usage

```sh
stencila snap [OPTIONS] [PATH] [OUTPUT]
```

# Examples

```bash
# Start server in background
stencila serve --sync in &

# Capture viewport screenshot (default)
stencila snap snaps/viewport.png

# Capture full scrollable page
stencila snap --full snaps/full.png

# Verify computed padding for title
stencila snap --assert "css([slot=title]).paddingTop>=24px"

# Capture mobile viewport of specific element
stencila snap --device mobile --selector "stencila-article [slot=title]" snaps/mobile.png

# Capture full mobile page
stencila snap --device mobile --full snaps/mobile-full.png

# Force light or dark mode
stencila snap --light snaps/light.png
stencila snap --dark snaps/dark.png

# Preview with PDF/print styles (A4 width)
stencila snap --print snaps/print-preview.png

# Multiple assertions without screenshot
stencila snap \
--assert "css([slot=title]).fontSize>=28px" \
--assert "count(section)==5" \
--measure

# Use custom viewport and wait conditions
stencila snap \
--width 1920 --height 1080 \
--wait-until networkidle \
--delay 500 \
snaps/desktop.png

# Capture specific document path
stencila snap docs/guide.md snaps/guide.png
```

# Arguments

| Name       | Description                    |
| ---------- | ------------------------------ |
| `[PATH]`   | Path to document or directory. |
| `[OUTPUT]` | Output screenshot path (.png). |

# Options

| Name                        | Description                                                                                                                                                                                                                                         |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--selector`                | CSS selector to capture or measure.                                                                                                                                                                                                                 |
| `--full <FULL>`             | Capture full scrollable page. Possible values: `true`, `false`.                                                                                                                                                                                     |
| `--device <DEVICE>`         | Device preset. Possible values: `laptop` (Laptop (1440x900 @2x DPR)), `desktop` (Desktop (1920x1080 @1x DPR)), `mobile` (Mobile (390x844 @3x DPR)), `tablet` (Tablet (768x1024 @2x DPR)), `tablet-landscape` (Tablet Landscape (1024x768 @2x DPR)). |
| `--width`                   | Viewport width in pixels.                                                                                                                                                                                                                           |
| `--height`                  | Viewport height in pixels.                                                                                                                                                                                                                          |
| `--dpr`                     | Device pixel ratio.                                                                                                                                                                                                                                 |
| `--light <LIGHT>`           | Use light color scheme. Possible values: `true`, `false`.                                                                                                                                                                                           |
| `--dark <DARK>`             | Use dark color scheme. Possible values: `true`, `false`.                                                                                                                                                                                            |
| `--print <PRINT>`           | Preview with print media styles (A4 width, for PDF preview). Possible values: `true`, `false`.                                                                                                                                                      |
| `--wait-until <WAIT_UNTIL>` | When to capture: load, domcontentloaded, networkidle. Possible values: `load` (Wait for 'load' event), `dom-content-loaded` (Wait for 'DOMContentLoaded' event), `network-idle` (Wait for network idle (default)). Default value: `network-idle`.   |
| `--wait-for`                | Wait for CSS selector to exist before capturing.                                                                                                                                                                                                    |
| `--delay`                   | Additional delay in milliseconds after page is ready.                                                                                                                                                                                               |
| `--measure <MEASURE>`       | Collect computed CSS and layout metrics. Possible values: `true`, `false`.                                                                                                                                                                          |
| `--assert`                  | Assert measurement conditions.                                                                                                                                                                                                                      |
| `--url`                     | Override URL (instead of discovering server).                                                                                                                                                                                                       |
