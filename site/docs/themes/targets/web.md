---
title: Web Plotting Theme Target
description: |
  How plot theme tokens map into JavaScript-based renderers (Plotly, Vega-Lite, ECharts).
---

# Overview

In web and HTML outputs, Stencila reads `--plot-*` CSS variables from `:root` and builds a plot token bundle that JS renderers consume. Theme updates trigger plot recompilation in the browser.

This translation happens whenever Stencila renders interactive plots in the browser or HTML output, for example:

```sh
stencila render report.smd report.html
```

# Token Bundle

The plot token bundle is built in [`plotTheme.ts`](https://github.com/stencila/stencila/blob/main/web/src/utilities/plotTheme.ts) and passed to renderer adapters (e.g. Plotly, Vega-Lite, ECharts). If `--plot-theme: none` is set, renderers keep their defaults.

| Tokens | Web/JS output |
| --- | --- |
| `--plot-aspect-ratio`, `--plot-width`, `--plot-height`, `--plot-dpi` | Dimensions and rendering size |
| `--plot-height-min`, `--plot-height-max` | Container min/max sizing |
| `--plot-color-1` ... `--plot-color-12` | Categorical palette |
| `--plot-shape-1` ... `--plot-shape-8` | Marker shape palette |
| `--plot-line-type-1` ... `--plot-line-type-6` | Line type palette |
| `--plot-ramp-start`, `--plot-ramp-end` | Sequential color ramp |
| `--plot-background`, `--plot-panel` | Plot and panel backgrounds |
| `--plot-panel-border` | Panel border toggle |
| `--plot-grid-color`, `--plot-grid-width`, `--plot-grid-x-width`, `--plot-grid-y-width` | Grid styling |
| `--plot-text-color`, `--plot-font-family`, `--plot-font-size`, `--plot-title-size`, `--plot-subtitle-size` | Typography |
| `--plot-padding-*` | Plot padding |
| `--plot-axis-line-color`, `--plot-axis-line-width`, `--plot-axis-title-*` | Axis styling |
| `--plot-legend-*` | Legend background, border, text, position |
| `--plot-tooltip-*` | Tooltip background and text |
| `--plot-point-opacity`, `--plot-point-size`, `--plot-line-width`, `--plot-area-opacity` | Mark styling |

# Implementation Notes

- Plot tokens are read from CSS variables and cached; theme changes clear the cache and recompile plots.
- Renderer adapters live in [`image-object-plotly.ts`](https://github.com/stencila/stencila/blob/main/web/src/nodes/image-object-plotly.ts), [`image-object-vegalite.ts`](https://github.com/stencila/stencila/blob/main/web/src/nodes/image-object-vegalite.ts), and [`image-object-echarts.ts`](https://github.com/stencila/stencila/blob/main/web/src/nodes/image-object-echarts.ts).
- Theme change handling is wired in [`image-object.ts`](https://github.com/stencila/stencila/blob/main/web/src/nodes/image-object.ts).
- Plot theme presets are documented in [`tokens/plots.smd`](../tokens/plots.smd).
