---
title: R Theme Target
description: |
  How plot theme tokens map into R base graphics and ggplot2.
---

# Overview

R kernels receive Stencila `plot-*` tokens and apply them to both base graphics (`par()`) and ggplot2 theme defaults. Lengths are converted to points, and fonts are resolved before they are sent to the kernel.

This translation happens whenever Stencila executes R code that produces plots, for example during a [`render`](../../cli/render.md):

```sh
stencila render report.qmd report.html
```

# Theme Transfer

- Only `plot-*` tokens are sent to R kernels (see [`lib.rs`](https://github.com/stencila/stencila/blob/main/rust/kernel-micro/src/lib.rs)).
- Lengths are converted to points, and font stacks are resolved to installed font families.
- The R theme implementation lives in [`theme.r`](https://github.com/stencila/stencila/blob/main/rust/kernel-r/src/theme.r).

# Key Mappings

## Base Graphics (`par()`)

| Tokens | R output |
| --- | --- |
| `--plot-background` | `par(bg)` |
| `--plot-axis-line-color` | `par(fg)` |
| `--plot-tick-color` | `par(col.axis)` |
| `--plot-axis-title-color` | `par(col.lab)` |
| `--plot-axis-title-weight` | `par(font.lab)` |
| `--plot-text-color` | `par(col.main)`, `par(col.sub)` |
| `--plot-line-width` | `par(lwd)` |
| `--plot-font-family` | `par(family)` (mapped to `sans`, `serif`, `mono`) |
| `--plot-font-size` | `par(ps)` |
| `--plot-axis-title-size` | `par(cex.lab)` |
| `--plot-title-size` | `par(cex.main)` |
| `--plot-subtitle-size` | `par(cex.sub)` |
| `--plot-padding-*` | `par(mar)` (plot margins) |
| `--plot-panel-border` | `par(bty)` (box type) |

## ggplot2 Defaults

| Tokens | ggplot2 output |
| --- | --- |
| `--plot-panel` | `panel.background` |
| `--plot-panel-border` | `panel.border` toggle |
| `--plot-axis-line-color`, `--plot-axis-line-width` | `axis.line` |
| `--plot-grid-color`, `--plot-grid-x-width`, `--plot-grid-y-width` | `panel.grid.major/minor` |
| `--plot-axis-title-color`, `--plot-axis-title-size`, `--plot-axis-title-weight` | `axis.title.*` |
| `--plot-tick-color`, `--plot-tick-width`, `--plot-tick-size` | `axis.ticks`, `axis.ticks.length` |
| `--plot-font-family`, `--plot-font-size`, `--plot-text-color` | `text` (global); `plot.title`, `plot.subtitle` size/color |
| `--plot-title-size`, `--plot-subtitle-size` | `plot.title`, `plot.subtitle` |
| `--plot-padding-*` | `plot.margin` |
| `--plot-legend-background`, `--plot-legend-border-*`, `--plot-legend-text-color`, `--plot-legend-size`, `--plot-legend-position` | `legend.background`, `legend.key`, `legend.text`, `legend.title`, `legend.position` |

## Palettes and Scales

| Tokens | R output |
| --- | --- |
| `--plot-color-1` ... `--plot-color-12` | `palette()` and ggplot2 discrete scales |
| `--plot-ramp-start`, `--plot-ramp-end` | Gradient for continuous scales |
| `--plot-shape-1` ... `--plot-shape-8` | Base point symbols and ggplot2 shape scale |
| `--plot-line-type-1` ... `--plot-line-type-6` | Base line types and ggplot2 linetype scale |
| `--plot-point-opacity`, `--plot-point-size` | Default point alpha/size (ggplot2) |

# Not Yet Mapped

Some tokens are noted in [`theme.r`](https://github.com/stencila/stencila/blob/main/rust/kernel-r/src/theme.r) but not yet applied, including `plot-grid-dash`, `plot-legend-gap`, `plot-legend-marker-size`, and several interaction/animation tokens (e.g. `plot-tooltip-*`, `plot-crosshair-*`, `plot-anim-*`).

Plot theme presets are documented in [`tokens/plots.smd`](../tokens/plots.smd).
