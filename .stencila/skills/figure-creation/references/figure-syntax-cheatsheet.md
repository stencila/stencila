---
title: Figure Syntax Cheatsheet
description: Quick-reference for Stencila Markdown figure syntax — fences, captions, labels, executable figures, subfigure indentation, and cross-references.
---

## Basic figure

```smd
::: figure <label> #<id> [<layout>] {pad="<padding>"}

![](image.png)

The caption text.

:::
```

Content vs caption: paragraphs containing only an image/audio/video become **content**; all other paragraphs become **caption**. Caption can appear before or after content.

## Labels, IDs, and cross-references

- Auto-numbered labels: "Figure 1", "Figure 2", etc.
- Subfigures: "Figure 1A", "Figure 1B", etc.
- Auto-derived IDs: `fig-1`, `fig-1a`, `fig-2`, etc.
- Link syntax: `[Figure 1](#fig-1)` or `[](#fig-1)` (auto-filled).
- Avoid explicit labels — auto-numbering stays correct on reorder.

## Stable IDs

Use `#<id>` on the fence line for a stable, human-readable ID that survives reordering:

```smd
::: figure #specimen-1
```

The `#id` can appear in any position relative to label, layout, and attributes. Prefer stable IDs when the figure is referenced externally or when using `snap` to target a specific figure (e.g. `selector: "[id='specimen-1']"`).

## Executable figure

Wrap an executable code block inside the figure fence. The code output becomes the figure content.

````smd
::: figure

```plotly exec
{
  "data": [{"type": "bar", "x": ["A","B"], "y": [4,7]}],
  "layout": {"xaxis": {"title": {"text": "Category"}}}
}
```

A bar chart caption.

:::
````

Works with any kernel: `plotly`, `r`, `python`, `node`, etc.

## Subfigures

Nested `::: figure` blocks **must be indented 4 spaces** inside the parent:

```smd
::: figure

    ::: figure

    ![](panel-a.png)

    Panel A caption.

    :::

    ::: figure

    ![](panel-b.png)

    Panel B caption.

    :::

Overall caption.

:::
```

## Multi-panel layout mini-language

Add layout in square brackets after `figure`:

| Pattern | Result |
|---|---|
| _(none)_ | Vertical stack |
| `[row]` | Single row |
| `[2]` or `[3]` | Equal-width column grid |
| `[30 70]` | Proportional column widths |
| `[40 g20 40]` | Two columns with gap (`g` prefix) |
| `[a b \| a c]` | Layout map — `a` spans two rows left |
| `[a a \| b c]` | Layout map — `a` spans two columns top |
| `[a . \| b c]` | `.` = empty cell |
| `[30 70 : a b \| a c]` | Column widths + layout map |

Letters map to subfigures in order (`a` = first, `b` = second, …). Pipes separate rows. Repeating a letter across rows/columns creates a spanning subfigure.

## Padding

Add whitespace around content with `{pad="..."}`:

```smd
::: figure {pad=50}
::: figure {pad="30 60"}
::: figure [2] {pad="10 20 30 40"}
```

- `50` — all sides
- `"30 60"` — vertical horizontal
- `"10 20 30 40"` — top right bottom left

Quote multi-value padding. Padding extends the overlay coordinate space:

```
viewBox with padding (for image W×H, pad="T R B L"):
  viewBox = "0 0 (W+L+R) (H+T+B)"
  Image top-left is at coordinate (L, T)
  Image-space coordinates shift by +L, +T

Example: 600×300 image, pad="0 0 56 0"
  → viewBox="0 0 600 356"  (top=0, left=0 so image coords unchanged)

Example: 600×300 image, pad="50 20 56 20"
  → viewBox="0 0 640 406"  (image top-left at 20,50 — coords shift +20,+50)
```

## Overlay block

```smd
::: figure

![](image.png)

```svg overlay
<svg viewBox="0 0 600 300" xmlns:s="https://stencila.io/svg">
  <s:callout x="300" y="100" label="Note" to-x="200" to-y="180"/>
</svg>
```

Caption text.

:::
```

Key rules:
- Always declare `xmlns:s="https://stencila.io/svg"` when using `<s:*>` components.
- `viewBox` defines the coordinate system; matching image dimensions is convenient but not required.
- For browser-rendered charts (Plotly, eCharts), set explicit `width`/`height` in chart config and use matching `viewBox`.
- Each subfigure can have its own overlay. Parent figures can also have a grid-spanning overlay.
- Parent overlays auto-hide on small screens when the grid collapses.
