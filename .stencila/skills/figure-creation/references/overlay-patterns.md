---
title: Overlay Patterns
description: Common overlay annotation patterns for Stencila figures — anchors, callouts, ROIs, measurement, and mixed component/raw SVG recipes.
---

## Anchor-based positioning (preferred)

Use named anchors to avoid coordinate duplication. When a feature moves, update one anchor instead of multiple components.

```xml
<svg viewBox="0 0 600 300" xmlns:s="https://stencila.io/svg">
  <s:anchor id="feature" x="200" y="120"/>
  <s:halo at="#feature" r="15" width="8"/>
  <s:marker at="#feature" symbol="circle" size="12"/>
  <s:callout from="#feature" dx="150" dy="-60" label="Key finding" to="#feature"/>
</svg>
```

## Auto-anchor positioning

Use built-in auto-anchors for common edge/corner placements:

| Anchor | Position |
|---|---|
| `#s:center` | Center of viewBox |
| `#s:top-left`, `#s:top-right` | Top corners |
| `#s:bottom-left`, `#s:bottom-right` | Bottom corners |
| `#s:top-center`, `#s:bottom-center` | Edge midpoints |
| `#s:mid-left`, `#s:mid-right` | Side midpoints |

```xml
<s:scale-bar at="#s:bottom-left" dx="40" dy="-30" length="130" label="20 μm"/>
<s:compass at="#s:bottom-right" dx="-40" dy="-30" size="36"/>
<s:badge at="#s:top-left" dx="20" dy="20" label="A"/>
```

## Callout with leader line

```xml
<s:callout x="400" y="60" label="Annotation text" to-x="250" to-y="150" curve="quad" shape="rect"/>
```

Curve types: `straight` (default), `quad`, `cubic`, `elbow`.

## Region of interest highlighting

```xml
<!-- Rectangle -->
<s:roi-rect x="50" y="40" width="140" height="90" label="Region A" stroke-style="dashed"/>

<!-- Ellipse -->
<s:roi-ellipse cx="300" cy="150" rx="80" ry="50" label="Region B"/>

<!-- Polygon -->
<s:roi-polygon points="100,60 250,40 300,120 220,200 80,160" label="Boundary"/>

<!-- Spotlight (inverse highlight — dims everything outside) -->
<s:spotlight cx="200" cy="150" r="70"/>
```

## Arrows and connectors

```xml
<!-- Straight with label -->
<s:arrow x="100" y="100" to-x="300" to-y="200" label="Direction"/>

<!-- Curved -->
<s:arrow x="100" y="250" to-x="300" to-y="50" curve="quad" label="Trend"/>

<!-- Elbow connector -->
<s:arrow x="80" y="60" to-x="280" to-y="240" curve="elbow" corner="horizontal-first"/>

<!-- Between anchors -->
<s:arrow from="#start" to="#end" curve="quad" tip="both" label="Bidirectional"/>
```

## Measurement annotations

```xml
<!-- Scale bar -->
<s:scale-bar x="40" y="280" length="130" label="50 μm"/>

<!-- Dimension line -->
<s:dimension x="100" y="200" to-x="400" to-y="200" label="15.2 mm" side="above"/>

<!-- Angle arc -->
<s:angle x="200" y="200" from-x="300" from-y="200" to-x="250" to-y="120" r="60" label="45°"/>

<!-- Compass / north arrow -->
<s:compass at="#s:bottom-right" dx="-50" dy="-50" size="40"/>
```

## Grouping with braces and brackets

```xml
<!-- Curly brace -->
<s:brace x="100" y="250" to-x="400" to-y="250" side="below" label="Treatment group"/>

<!-- Statistical bracket -->
<s:bracket x="150" y="50" to-x="350" to-y="50" side="above" label="p < 0.01" variant="square"/>
```

## Point markers

```xml
<s:marker x="150" y="100" symbol="pin" label="Site 1" size="24"/>
<s:marker x="300" y="180" symbol="star" label="Site 2" size="22" color="#ff6600"/>
<s:marker x="500" y="120" symbol="circle" fill="red" stroke="none" size="16"/>
<s:crosshair cx="400" cy="120" size="25" gap="5" ring="true" label="Target"/>
```

Symbols: `circle`, `cross`, `diamond`, `pin`, `plus`, `square`, `star`, `triangle`, `triangle-down`. Use `color` to set fill, stroke, and text; use `fill`/`stroke`/`text` individually to override. Default: `currentColor`.

## Subfigure overlay pattern

Each subfigure gets its own overlay scoped to that panel:

`````smd
::: figure [2]

    ::: figure

    ![](left.png)

    ```svg overlay
    <svg viewBox="0 0 400 250" xmlns:s="https://stencila.io/svg">
      <s:roi-rect x="50" y="40" width="140" height="90" label="Region A"/>
    </svg>
    ```

    Left panel.

    :::

    ::: figure

    ![](right.png)

    ```svg overlay
    <svg viewBox="0 0 400 250" xmlns:s="https://stencila.io/svg">
      <s:roi-ellipse cx="200" cy="125" rx="80" ry="50" label="Region B"/>
    </svg>
    ```

    Right panel.

    :::

Two panels with individual annotations.

:::
`````

## Parent overlay connecting subfigures

A parent overlay spans the entire grid — useful for cross-panel annotations:

```xml
<!-- Parent overlay (viewBox covers full grid) -->
<svg viewBox="0 0 800 250" xmlns:s="https://stencila.io/svg">
  <s:arrow x="350" y="150" to-x="500" to-y="150" label="Leads to"/>
</svg>
```

Parent overlays auto-hide when the grid collapses on small screens.

## Custom colors

All components support `stroke`, `fill`, `text`, and `color` attributes. The `color` shorthand sets stroke, fill, and text; explicit `fill`, `stroke`, or `text` override it. Arrow markers automatically match the stroke of the line they are attached to.

```xml
<!-- Colored arrow — arrowhead matches automatically -->
<s:arrow x="100" y="100" to-x="300" to-y="200" stroke="crimson" stroke-width="2"/>

<!-- Styled callout with custom background, border, and text color -->
<s:callout x="300" y="160" label="n = 1,024" shape="pill" fill="#e8f4fd" stroke="#4a90d9" text="#1a5276"/>

<!-- Color shorthand sets fill, stroke, and text on marker -->
<s:marker x="200" y="100" symbol="circle" color="#2563eb"/>

<!-- Colored ROI — color shorthand sets both stroke and label text -->
<s:roi-rect x="50" y="40" width="140" height="90" label="Region A" color="green" stroke-style="dashed"/>

<!-- Dimension line with different stroke and text colors -->
<s:dimension x="100" y="300" to-x="400" to-y="300" label="4.2 cm" stroke="gray" text="navy"/>
```

## Mixing components with raw SVG

Components and standard SVG work together in the same overlay:

```xml
<svg viewBox="0 0 600 300" xmlns:s="https://stencila.io/svg">
  <!-- Raw SVG for a custom shape -->
  <rect x="240" y="100" width="120" height="100" rx="12"
        fill="none" stroke="currentColor" stroke-dasharray="6 3"/>

  <!-- Component callout pointing at it -->
  <s:callout x="480" y="50" label="Custom region" to-x="300" to-y="150" curve="quad"/>

  <!-- Component scale bar -->
  <s:scale-bar at="#s:bottom-left" dx="30" dy="-30" length="100" label="10 mm"/>
</svg>
```
