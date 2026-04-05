---
title: Figure Syntax Cheatsheet
description: Quick-reference for Stencila Markdown figure syntax — fences, captions, labels, executable figures, subfigure indentation, and cross-references.
---

## Basic figure

```smd
::: figure

![](image.png)

The caption text.

:::
```

Content vs caption: paragraphs containing only an image/audio/video become **content**; all other paragraphs become **caption**. Caption can appear before or after content.

## Labels and cross-references

- Auto-numbered: "Figure 1", "Figure 2", etc.
- Subfigures: "Figure 1A", "Figure 1B", etc.
- IDs: `fig-1`, `fig-1a`, `fig-2`, etc.
- Link syntax: `[Figure 1](#fig-1)` or `[](#fig-1)` (auto-filled).
- Avoid explicit labels — auto-numbering stays correct on reorder.

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

Quote multi-value padding. Padding extends the overlay coordinate space.

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
