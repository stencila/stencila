---
title: Overlay Components Quick Reference
description: Condensed attribute tables for all Stencila SVG overlay components, organized by category.
---

All components use the `s:` namespace prefix. Declare `xmlns:s="https://stencila.io/svg"` on the `<svg>` element.

## Common positioning attributes

| Attribute | Description |
|---|---|
| `x`, `y` | Direct coordinates in viewBox units |
| `at` | Anchor reference for single-point components (e.g., `at="#s:bottom-left"`) |
| `from` | Start anchor for two-point components |
| `to` | End anchor for two-point components |
| `to-x`, `to-y` | Direct end coordinates |
| `dx`, `dy` | Offset from anchor or position |

Single-point components (`marker`, `compass`, `halo`, `crosshair`, `spotlight`, `badge`) use `at`. Two-point components (`arrow`, `brace`, `bracket`, `dimension`, `roi-rect`) use `from`/`to`.

## Common connector attributes

| Attribute | Values | Default |
|---|---|---|
| `curve` | `straight`, `quad`, `cubic`, `elbow` | `straight` |
| `corner` | `horizontal-first`, `vertical-first` | `horizontal-first` |
| `tip` | `end`, `start`, `both`, `none` | `end` |
| `tip-style` | `s:arrow-closed`, `s:arrow-open`, `s:arrow-dot` | `s:arrow-closed` |

## Arrow (`<s:arrow>`)

Directional line/curve between two points. Required: start (`x,y` or `from`) + end (`to-x,to-y` or `to`).

| Attribute | Values | Default |
|---|---|---|
| `curve` | `straight`, `quad`, `cubic`, `elbow` | `straight` |
| `tip` | `end`, `start`, `both`, `none` | `end` |
| `label` | text | — |
| `label-position` | `above`, `below` | `above` |
| `label-angle` | `along`, `horizontal`, degrees | `along` |

## Callout (`<s:callout>`)

Text label with optional background shape and leader line. Required: position (`x,y` or `from`) + `label`.

| Attribute | Values | Default |
|---|---|---|
| `label` | text | — |
| `shape` | `none`, `rect`, `pill` | `none` |
| `to-x,to-y` or `to` | target for leader line | — |
| `curve` | `straight`, `quad`, `cubic`, `elbow` | `straight` |
| `tip` | `end`, `start`, `both`, `none` | `end` |
| `text-anchor` | `start`, `middle`, `end` | `start` |

## Badge (`<s:badge>`)

Compact pill-shaped label (1–4 chars ideal). Required: position + `label`.

| Attribute | Values | Default |
|---|---|---|
| `label` | text (short) | — |
| `fill` | color | theme default |

## Scale bar (`<s:scale-bar>`)

Calibrated measurement bar. Required: position + `length`.

| Attribute | Values | Default |
|---|---|---|
| `length` | number (viewBox units) | — |
| `label` | text (e.g., "20 μm") | — |
| `side` | `above`, `below` | `below` |

## Dimension line (`<s:dimension>`)

Engineering-style dimension with end caps. Required: start + end positions.

| Attribute | Values | Default |
|---|---|---|
| `label` | text | — |
| `side` | `above`, `below` | `above` |
| `gap` | number | `0` |

## Angle arc (`<s:angle>`)

Arc between two rays from a vertex. Required: vertex (`x,y`) + two ray endpoints.

| Attribute | Values | Default |
|---|---|---|
| `from-x,from-y` | first ray endpoint | — |
| `to-x,to-y` | second ray endpoint | — |
| `r` | arc radius | `40` |
| `label` | text (e.g., "45°") | — |

## Brace (`<s:brace>`)

Curly brace for grouping. Required: start + end positions.

| Attribute | Values | Default |
|---|---|---|
| `side` | `above`, `below`, `left`, `right` | `below` |
| `label` | text | — |

## Bracket (`<s:bracket>`)

Square/round bracket (e.g., significance markers). Required: start + end positions.

| Attribute | Values | Default |
|---|---|---|
| `side` | `above`, `below`, `left`, `right` | `above` |
| `variant` | `square`, `round` | `square` |
| `label` | text (e.g., "p < 0.05") | — |

## ROI rectangle (`<s:roi-rect>`)

Rectangular region of interest. Required: position + `width` + `height` (or `from`/`to`).

| Attribute | Values | Default |
|---|---|---|
| `width`, `height` | dimensions | — |
| `label` | text | — |
| `label-position` | `above`, `below`, `center`, `left`, `right` | `above` |
| `stroke-style` | `solid`, `dashed`, `dotted` | `solid` |

## ROI ellipse (`<s:roi-ellipse>`)

Elliptical region of interest. Required: center (`cx,cy` or `at`) + radii.

| Attribute | Values | Default |
|---|---|---|
| `cx`, `cy` | center coordinates | — |
| `rx`, `ry` | radii | — |
| `label` | text | — |
| `label-position` | `above`, `below`, `center`, `left`, `right` | `above` |
| `stroke-style` | `solid`, `dashed`, `dotted` | `solid` |

## ROI polygon (`<s:roi-polygon>`)

Polygonal region. Required: `points`.

| Attribute | Values | Default |
|---|---|---|
| `points` | space-separated `x,y` pairs | — |
| `label` | text | — |
| `stroke-style` | `solid`, `dashed`, `dotted` | `solid` |

## Spotlight (`<s:spotlight>`)

Inverse highlight — dims everything outside a region. Required: center position.

| Attribute | Values | Default |
|---|---|---|
| `r` | radius (circular) | `50` |
| `rx`, `ry` | radii (elliptical) | — |
| `shape` | `circle`, `rect` | `circle` |
| `width`, `height` | dimensions (rect) | `100` |
| `opacity` | `0.0`–`1.0` (dimmed area) | `0.6` |

## Marker (`<s:marker>`)

Symbol glyph at a location. Required: position.

| Attribute | Values | Default |
|---|---|---|
| `symbol` | `circle`, `cross`, `diamond`, `pin`, `plus`, `square`, `star`, `triangle`, `triangle-down` | `circle` |
| `size` | number | `20` |
| `color` | color | `currentColor` |
| `fill` | color | `currentColor` |
| `stroke` | color | `currentColor` |
| `background` | color or `none` | `white` |
| `label` | text | — |
| `label-position` | `right`, `above`, `below`, `left` | `right` |

`color` sets both fill and stroke. Use `fill` or `stroke` individually to override one while keeping the other. Use `fill="none"` for outline-only markers. A 50% opacity `background` (default white) is rendered behind the symbol and label for legibility; set `background="none"` to disable.

## Crosshair (`<s:crosshair>`)

Reticle at a precise location. Required: center position.

| Attribute | Values | Default |
|---|---|---|
| `size` | arm length | `20` |
| `gap` | gap radius around center | `4` |
| `ring` | `true`, `false` | `false` |
| `label` | text | — |

## Halo (`<s:halo>`)

Semi-transparent glowing ring. Required: center position.

| Attribute | Values | Default |
|---|---|---|
| `r` | inner radius | `15` |
| `width` | ring thickness | `8` |
| `color` | ring color | `currentColor` |
| `opacity` | `0.0`–`1.0` | `0.4` |

## Compass (`<s:compass>`)

Orientation indicator. Required: position.

| Attribute | Values | Default |
|---|---|---|
| `size` | overall size | `50` |
| `variant` | `arrow`, `full` | `arrow` |
| `axes` | axis labels (e.g., `"A/P D/V"`) | `N/S E/W` |

## Built-in SVG definitions

Arrow markers (for `tip-style` or raw SVG `marker-end`):
- `s:arrow-closed` — filled triangle (default)
- `s:arrow-open` — open chevron
- `s:arrow-dot` — filled circle
- `s:cap-line` — perpendicular cap

Marker symbols (for `<s:marker>` `symbol` or `<use href="#s:marker-*">`):
- `s:marker-circle`, `s:marker-cross`, `s:marker-diamond`, `s:marker-pin`, `s:marker-plus`, `s:marker-square`, `s:marker-star`, `s:marker-triangle`, `s:marker-triangle-down`
