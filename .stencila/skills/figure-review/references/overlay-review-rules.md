---
title: Overlay Review Rules
description: Condensed review rules for Stencila SVG figure overlays, annotation components, measurement safety, and rendering risks.
---

## Basic overlay structure

```smd
```svg overlay
<svg viewBox="0 0 600 300" xmlns:s="https://stencila.io/svg">
  <s:callout x="300" y="100" label="Note" to-x="200" to-y="180"/>
</svg>
```
```

Review checks:

- `xmlns:s="https://stencila.io/svg"` is required when using `<s:*>` components.
- The `viewBox` defines the coordinate system; all overlay geometry must be coherent within that system.
- Parent-level overlays are appropriate for cross-panel annotations; subfigure-local overlays are better for panel-specific annotations.

## Prefer maintainable positioning

Anchor-based positioning is usually more maintainable than repeating raw coordinates.

```xml
<s:anchor id="peak" x="250" y="80"/>
<s:halo at="#peak" r="15" width="8"/>
<s:callout from="#peak" dx="150" dy="-60" label="Maximum" to="#peak"/>
```

Review checks:

- If multiple annotations target the same feature, recommend a named anchor.
- Repeated raw coordinates make overlays brittle during later edits.

## High-value component checks

| Component | Review focus |
|---|---|
| `<s:callout>` | label clarity, leader-line target, overlap risk |
| `<s:arrow>` | source/target correctness, curve readability, clutter |
| `<s:roi-rect>`, `<s:roi-ellipse>`, `<s:roi-polygon>` | region accuracy, label placement, unnecessary clutter |
| `<s:scale-bar>` | known calibration for `length` and `label` |
| `<s:dimension>` | supported measurement rather than guessed value |
| `<s:compass>` | meaningful and known axis semantics |
| `<s:badge>` | short semantic token, not duplicate panel lettering |
| `<s:halo>`, `<s:spotlight>`, `<s:crosshair>` | visual emphasis without obscuring content |

## Measurement safety

Treat these as serious review concerns:

- scale bars with no known calibration
- dimension or angle labels unsupported by supplied measurements
- compass indicators with unknown or unjustified orientation semantics
- precise annotation claims that are not supported by the source or rendering context

When calibration or orientation data is missing, recommend asking for it rather than approving the figure.

## Rendering risks to check with `snap`

Use `snap` when possible to confirm:

- label collisions or overlapping callouts
- overlays extending beyond the visible figure region
- cropped captions or layout overflow
- executable-chart overlays drifting out of alignment because chart `width`/`height` are not fixed
- parent overlays becoming misleading when a multi-panel layout collapses on small screens

Without `snap`, report these as unverified rendering risks rather than confirmed defects.
