---
title: Python Theme Target
description: |
  How plot theme tokens map into matplotlib rcParams.
---

# Overview

Python kernels translate Stencila `plot-*` tokens into matplotlib `rcParams`. Lengths are converted to points, and fonts are resolved before being sent to the kernel.

This translation happens whenever Stencila executes Python code that produces plots, for example during a [`render`](../../cli/render.md):

```sh
stencila render report.ipynb report.html
```

# Theme Transfer

- Only `plot-*` tokens are sent to Python kernels (see [`lib.rs`](https://github.com/stencila/stencila/blob/main/rust/kernel-micro/src/lib.rs)).
- Lengths are converted to points, then mapped into matplotlib units (inches for figure size, points for text).
- The Python theme implementation lives in [`theme.py`](https://github.com/stencila/stencila/blob/main/rust/kernel-python/src/theme.py).

# Key Mappings

| Tokens | Matplotlib rcParams |
| --- | --- |
| `--plot-background` | `figure.facecolor`, `savefig.facecolor` |
| `--plot-panel` | `axes.facecolor` |
| `--plot-panel-border` | `axes.spines.*` (false = left/bottom only) |
| `--plot-axis-line-color` | `axes.edgecolor` |
| `--plot-axis-line-width` | `axes.linewidth` |
| `--plot-axis-title-color` | `axes.labelcolor` |
| `--plot-axis-title-size` | `axes.labelsize` |
| `--plot-axis-title-weight` | `axes.labelweight` |
| `--plot-title-size` | `axes.titlesize` |
| `--plot-text-color` | `text.color`, `axes.titlecolor` |
| `--plot-font-family` | `font.family` |
| `--plot-font-size` | `font.size` |
| `--plot-width`, `--plot-height` | `figure.figsize` (pt to in) |
| `--plot-dpi` | `figure.dpi`, `savefig.dpi` |
| `--plot-padding-*` | `figure.constrained_layout.h_pad` / `w_pad` (max of top/bottom, left/right) |
| `--plot-grid-color` | `grid.color` |
| `--plot-grid-x-width`, `--plot-grid-y-width`, `--plot-grid-width` | `axes.grid`, `grid.linewidth`, `axes.grid.axis` |
| `--plot-line-width` | `lines.linewidth` |
| `--plot-point-size` | `lines.markersize` |
| `--plot-point-opacity` | `axes.prop_cycle` alpha + `markers.fillstyle` |
| `--plot-color-1` ... `--plot-color-12` | `axes.prop_cycle` colors |
| `--plot-shape-1` ... `--plot-shape-8` | `axes.prop_cycle` markers |
| `--plot-line-type-1` ... `--plot-line-type-6` | `axes.prop_cycle` linestyles |
| `--plot-ramp-start`, `--plot-ramp-end` | `image.cmap` (generated gradient) |

# Notes and Gaps

- `--plot-subtitle-size` is not currently mapped in matplotlib.
- If `--plot-theme: none` is set, kernels skip theming.
- The authoritative mapping lives in [`theme.py`](https://github.com/stencila/stencila/blob/main/rust/kernel-python/src/theme.py).
