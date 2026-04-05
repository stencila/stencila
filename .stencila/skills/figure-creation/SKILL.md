---
name: figure-creation
description: Create, edit, or plan figures in Stencila Markdown — simple image figures, executable code figures, multi-panel subfigure layouts with grid arrangements, and SVG annotation overlays using overlay components. Use when asked to add or revise a figure, chart, plot, caption, subfigure grid, panel layout, overlay annotation, callout, scale bar, arrow, region-of-interest highlight, or figure design plan for a Stencila document.
keywords:
  - figure
  - image
  - chart
  - plot
  - caption
  - label
  - subfigure
  - multi-panel
  - grid layout
  - overlay
  - annotation
  - callout
  - arrow
  - scale bar
  - region of interest
  - ROI
  - halo
  - marker
  - brace
  - bracket
  - compass
  - dimension
  - spotlight
  - crosshair
  - badge
  - SVG
  - executable figure
  - plotly
  - smd
  - stencila markdown
  - figure plan
  - figure specification
allowed-tools: read_file write_file apply_patch glob grep snap ask_user
---

## Overview

Create, edit, or plan figures in Stencila Markdown (`.smd`) documents. Figures wrap images or executable output with captions, auto-numbered labels, cross-references, multi-panel grid layouts, and SVG annotation overlays.

If the user asks for strategy, options, or a figure specification rather than immediate file edits, provide a plan that covers the recommended figure structure, layout, caption approach, overlay strategy, and any missing inputs. Do not imply that implementation has already happened.

### References

Condensed references for quick lookup (try these first):

- [`references/figure-syntax-cheatsheet.md`](references/figure-syntax-cheatsheet.md) — figure syntax, labels, layouts, padding, and overlay block structure
- [`references/layout-cookbook.md`](references/layout-cookbook.md) — copy-paste multi-panel layout patterns
- [`references/overlay-patterns.md`](references/overlay-patterns.md) — common overlay annotation recipes and anchor patterns
- [`references/overlay-components-quick-ref.md`](references/overlay-components-quick-ref.md) — condensed attribute tables for all `<s:*>` components

Full documentation (large — load only when the condensed references are insufficient):

- [`references/figures.smd`](references/figures.smd) — complete figures documentation (763 lines)
- [`references/figure-overlay-components.smd`](references/figure-overlay-components.smd) — full overlay component reference with demos (980 lines)
- [`references/snap-tool.md`](references/snap-tool.md) — full `snap` tool reference for visual verification

## Required Inputs

| Input | Required | Description |
|---|---|---|
| Target `.smd` file or figure location | Required | Which document and where in it the figure should go, or which existing figure should be revised |
| Source images, existing figure content, or executable code/data | Required | The content the figure will display or the current figure material to revise |
| Caption text or intent | Required | What the caption should say or convey |
| Annotation/overlay intent | Optional | What to annotate, highlight, or label |
| Panel order and layout preference | Optional | How subfigures should be arranged |
| Calibration or scale info | Optional | Known scale for scale bars (e.g., "130px = 20 μm") |
| Orientation semantics | Optional | Axis meanings for compass indicators (e.g., N/S, A/P D/V) |
| Image dimensions | Optional | Pixel dimensions if precise overlay coordinates matter |
| Delivery mode | Optional | Whether the user wants design/specification guidance or an actual `.smd` edit |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

| Output | Description |
|---|---|
| Figure plan or updated `.smd` content | Either a figure specification when the user wants guidance, or the document with the new or modified figure block when the user wants implementation |
| Assumptions or questions | Any missing information that needs user confirmation |
| Verification status | Whether visual verification was performed, pending, or not applicable |

## Core rules

- Read the target document before editing so you can match existing figure, caption, and cross-reference conventions.
- Reuse existing panel order, caption tone, and labeling style unless the user asks to change them.
- Prefer anchor-based overlays when multiple annotations refer to the same feature.
- Prefer built-in subfigure labels over manually adding overlay badges just to show panel letters.
- Do not invent scale, orientation, measurements, or precise coordinates.

## Steps

1. **Determine the delivery mode.**
   - If the user wants a concept, design direction, or figure specification, provide a figure plan.
   - If the user wants an actual figure added or revised in a document, make the `.smd` edit.
   - If one missing detail materially changes the result, ask a clarifying question before authoring.
2. **Determine figure type and overlay scope.** Identify which combination the user needs:
   - Simple image figure (one image + caption)
   - Executable figure (code block output + caption)
   - Multi-panel figure (subfigures with grid layout)
   - Any of the above with overlay annotations
   - Subfigure-local overlays, parent grid-spanning overlays, or both
3. **Ask the user when information is missing.** If any of the following are unspecified, ask before guessing:
   - Which image files or panel order to use
   - Exact annotation targets or positions
   - Caption text or tone
   - Whether overlays should be approximate or precise
   - Image dimensions (if precise overlay coordinates matter)
   - Scale calibration for scale bars
   - Orientation semantics for compass indicators
4. **Read the target document** to understand existing figures, numbering, nearby references, and style conventions.
5. **Produce the requested output.**
   - For a plan, provide a figure specification covering figure type, panel order, caption approach, overlay strategy, and required missing inputs.
   - For implementation, author the figure using the syntax patterns below.
6. **Add overlays** if requested — prefer anchor-based positioning over raw coordinates (see "Anchor-first positioning" below).
7. **Verify cross-references and panel labeling.**
   - Confirm figure references still point to the right target.
   - Avoid duplicating automatic subfigure labels with manual overlay badges unless the badge serves a different semantic purpose.
8. **Optionally verify rendering** with `snap` if a Stencila server and route are available (see "Visual verification" below).

## Do not invent measurements

This is the most important safety rule for overlay annotations:

- **Do not fabricate scale bar values** without known calibration. If the user has not provided a scale (e.g., "130px = 20 μm"), ask for it.
- **Do not invent dimension line measurements** from a screenshot or image unless the user explicitly provides a scale.
- **Do not add compass/north arrows** unless orientation is meaningful and known for the content.
- **Do not claim precise coordinates** for annotations unless you have reliable information about the image layout.

When calibration or orientation information is missing, either ask the user or use placeholder labels that clearly indicate the values need confirmation (e.g., `label="[calibrate: ? μm]"`).

## Figure syntax

A figure uses a colon fence with the keyword `figure`:

````smd
::: figure

![](image.png)

The caption text.

:::
````

Stencila auto-separates content from caption: paragraphs containing only an image (or audio/video) become **content**; all other paragraphs become **caption**. The caption can appear before or after the image.

### Labels and cross-references

Figures are auto-numbered ("Figure 1", "Figure 2", …). Subfigures get sub-labels (A, B, C, …). Link to a figure with `[Figure 1](#fig-1)` or `[](#fig-1)` (auto-filled). Avoid explicit labels — auto-numbering stays correct when figures are reordered.

### Panel labels vs overlay badges

Subfigures already get automatic panel labels such as A, B, and C. Do not add `<s:badge label="A">`, `<s:badge label="B">`, etc. just to recreate those built-in labels unless the user explicitly wants additional in-image tokens. Otherwise the rendered figure can end up with duplicate panel lettering.

### Executable figures

Wrap an executable code block inside the figure fence. The code output becomes the figure content:

`````smd
::: figure

````plotly exec
{
  "data": [{"type": "bar", "x": ["A","B","C"], "y": [4,7,2]}],
  "layout": {"xaxis": {"title": {"text": "Category"}}}
}
````

A bar chart of values by category.

:::
`````

This works with any execution kernel: `plotly`, `r`, `python`, `node`, etc.

## Multi-panel layouts

Nest `::: figure` blocks as subfigures. Subfigures must be indented (4 spaces) inside the parent. Add a layout in square brackets after `figure`:

| Pattern | Result |
|---|---|
| _(none)_ | Subfigures stack vertically |
| `[row]` | All subfigures in a single row |
| `[2]` or `[3]` | Equal-width column grid |
| `[30 70]` | Proportional column widths (30:70 split) |
| `[40 g20 40]` | Two columns with a gap (`g` prefix) |
| `[a b \| a c]` | Layout map — `a` spans two rows on left |
| `[a a \| b c]` | Layout map — `a` spans two columns on top |
| `[a . \| b c]` | `.` leaves a cell empty |
| `[30 70 : a b \| a c]` | Column widths combined with layout map |

In layout maps, letters map to subfigures in order (`a` = first, `b` = second, etc.). Pipes (`|`) separate rows. See [`references/layout-cookbook.md`](references/layout-cookbook.md) for copy-paste patterns.

## Overlays

An overlay is an `svg overlay` fenced code block inside a figure. It renders a transparent SVG layer on top of the figure content. Use overlay **components** (`<s:*>` namespace) instead of raw SVG — they handle arrowhead definitions, label positioning, and coordinate math automatically.

**Key rules:**
- Always declare `xmlns:s="https://stencila.io/svg"` on the `<svg>` element when using components.
- The `viewBox` defines the coordinate system. Matching image dimensions is convenient (coordinates map to pixels) but any viewBox works.
- For browser-rendered charts (Plotly, eCharts), set explicit `width`/`height` in the chart config and use a matching `viewBox`.
- Shapes and text inherit theme defaults. Override per-element with inline SVG attributes when needed.

### Choose the right overlay scope

- **Overlay inside a subfigure**: use when the annotation belongs to one panel and should use that panel's local coordinate system.
- **Overlay on the parent figure**: use when the annotation spans the grid, connects panels, or needs figure-level positioning.
- **Both**: use subfigure overlays for panel-local highlights and a parent overlay for cross-panel connectors or labels.

Prefer the narrowest scope that matches the semantics of the annotation.

### Anchor-first positioning

Prefer anchor-based positioning over raw `x`/`y` coordinates — it is more maintainable and less brittle:

```xml
<!-- Define named anchors for important features -->
<s:anchor id="peak" x="250" y="80"/>
<s:anchor id="valley" x="420" y="200"/>

<!-- Reference anchors instead of repeating coordinates -->
<s:halo at="#peak" r="15" width="8"/>
<s:callout from="#peak" dx="150" dy="-60" label="Maximum" to="#peak"/>
<s:arrow from="#peak" to="#valley" curve="quad" label="Transition"/>
```

Use auto-anchors for common placements: `at="#s:bottom-left"`, `at="#s:top-right"`, `at="#s:center"`, etc.

Benefits: moving a feature means updating one `<s:anchor>`, not every component that references it.

### Component quick reference

| Component | Key attributes | Use for |
|---|---|---|
| `<s:arrow>` | start → end, `curve`, `tip`, `label` | Directional lines and curves |
| `<s:callout>` | position, `label`, `shape`, target, `curve` | Text labels with optional leader line |
| `<s:badge>` | position, `label` | Compact pill-shaped tokens (1–4 chars) |
| `<s:scale-bar>` | position, `length`, `label` | Calibrated measurement bars |
| `<s:dimension>` | start → end, `label`, `side` | Engineering-style dimension lines |
| `<s:angle>` | vertex, two ray endpoints, `r`, `label` | Angle arcs |
| `<s:brace>` | start → end, `side`, `label` | Curly braces for grouping |
| `<s:bracket>` | start → end, `side`, `variant`, `label` | Square/round brackets (e.g., `p < 0.05`) |
| `<s:roi-rect>` | position, `width`, `height`, `label` | Rectangular ROI outlines |
| `<s:roi-ellipse>` | center, `rx`, `ry`, `label` | Elliptical ROI outlines |
| `<s:roi-polygon>` | `points`, `label` | Polygonal region outlines |
| `<s:spotlight>` | center, `r`, `opacity` | Inverse highlight (dims outside) |
| `<s:marker>` | position, `symbol`, `color`, `label` | Symbol glyphs: `circle`, `cross`, `diamond`, `pin`, `plus`, `square`, `star`, `triangle`, `triangle-down` |
| `<s:crosshair>` | center, `size`, `gap`, `ring`, `label` | Reticle at a precise location |
| `<s:halo>` | center, `r`, `width`, `color`, `opacity` | Semi-transparent glowing ring |
| `<s:compass>` | position, `size`, `variant`, `axes` | Orientation indicator |

For full attribute tables, see [`references/overlay-components-quick-ref.md`](references/overlay-components-quick-ref.md). For demos of every component, see [`references/figure-overlay-components.smd`](references/figure-overlay-components.smd).

### Padding

Add whitespace around content with `{pad="..."}` so overlays can place elements outside the image:

```smd
::: figure {pad="0 0 56 0"}
```

Values: `50` (all sides), `"30 60"` (vertical horizontal), `"10 20 30 40"` (top right bottom left). Quote multi-value padding. Padding extends the overlay coordinate space — e.g., `{pad="0 0 56 0"}` on a 600×300 image means the overlay `viewBox` should be `"0 0 600 356"`.

## Examples

### Simple image figure

````smd
::: figure

![](photo.jpg)

A photograph of the study site taken in September 2024.

:::
````

### Executable figure with anchor-based overlay

`````smd
::: figure

```r exec
hist(rnorm(1000), breaks=30, col='#b0d0e8', border='#7aaccc', main='')
```

```svg overlay
<svg viewBox="0 0 600 400" xmlns:s="https://stencila.io/svg">
  <s:anchor id="peak" x="335" y="100"/>
  <s:halo at="#peak" r="20" width="8" color="crimson" opacity="0.4"/>
  <s:callout from="#peak" dx="125" dy="-55" label="Peak near μ=0" to="#peak" curve="quad" fill="crimson"/>
</svg>
```

Distribution of 1000 random normal variates.

:::
`````

### Two-column layout with subfigure overlays

`````smd
::: figure [2]

    ::: figure

    ![](left-panel.png)

    ```svg overlay
    <svg viewBox="0 0 400 250" xmlns:s="https://stencila.io/svg">
      <s:anchor id="region-a" x="120" y="85"/>
      <s:roi-rect from="#region-a" dx="-70" dy="-45" width="140" height="90" label="Region A" stroke-style="dashed"/>
    </svg>
    ```

    Left panel.

    :::

    ::: figure

    ![](right-panel.png)

    ```svg overlay
    <svg viewBox="0 0 400 250" xmlns:s="https://stencila.io/svg">
      <s:roi-ellipse cx="200" cy="125" rx="80" ry="50" label="Region B"/>
    </svg>
    ```

    Right panel.

    :::

Two panels with individual overlay annotations.

:::
`````

## Visual verification

If a Stencila server and rendered route are available, optionally use `snap` to verify overlay placement and layout:

```
snap(route: "/path/to/document", screenshot: true, selector: "stencila-figure")
```

If `snap` is unavailable, mark visual verification as pending. Do not claim rendered correctness unless `snap` was actually run.

For details on snap parameters and usage patterns, see [`references/snap-tool.md`](references/snap-tool.md).

## Edge cases

- **Subfigure indentation**: subfigure `::: figure` blocks must be indented 4 spaces inside the parent. Missing indentation makes them sibling figures, not subfigures.
- **Duplicate panel lettering**: subfigures already get automatic A/B/C labels. Do not add matching overlay badges unless the user explicitly wants separate in-image labels.
- **Executable + overlay alignment**: for Plotly/eCharts/Vega-Lite, pin `width` and `height` in the chart config and match the overlay `viewBox`. Dynamic resizing can misalign overlays.
- **Parent overlay on mobile**: parent-level overlays (spanning the grid) auto-hide when the grid collapses to vertical on small screens. Subfigure overlays are unaffected.
- **viewBox coordinate system**: the `viewBox` can use any coordinate system — matching image pixel dimensions is convenient but not required. All annotation geometry must use the same coordinate space as the `viewBox`.
- **Multi-value padding**: quote the `pad` value when using more than one number: `{pad="30 60"}`, not `{pad=30 60}`.
- **Cross-reference IDs**: IDs use a `fig-` prefix with the label lowercased — `fig-1`, `fig-2a`. If the user provides an explicit label, the ID derives from it.
- **Namespace declaration**: forgetting `xmlns:s="https://stencila.io/svg"` on the `<svg>` element causes components to be treated as unknown elements and silently ignored.
- **Mixing components and raw SVG**: components and standard SVG elements work together in the same overlay. Use components for high-level annotations and raw SVG when precise control is needed.
