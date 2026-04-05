---
title: Layout Cookbook
description: Common multi-panel figure layouts in Stencila Markdown with copy-paste examples.
---

## Two equal columns

```smd
::: figure [2]

    ::: figure

    ![](left.png)

    Left panel.

    :::

    ::: figure

    ![](right.png)

    Right panel.

    :::

Caption for both panels.

:::
```

## Three panels in a row

```smd
::: figure [row]

    ::: figure

    ![](a.png)

    Panel A.

    :::

    ::: figure

    ![](b.png)

    Panel B.

    :::

    ::: figure

    ![](c.png)

    Panel C.

    :::

Three panels side by side.

:::
```

## Unequal column widths (30:70)

```smd
::: figure [30 70]

    ::: figure

    ![](narrow.png)

    Narrow panel.

    :::

    ::: figure

    ![](wide.png)

    Wide panel.

    :::

Unequal split.

:::
```

## Layout map: tall panel spanning rows

```smd
::: figure [a b | a c]

    ::: figure

    ![](tall.png)

    Spans both rows.

    :::

    ::: figure

    ![](top-right.png)

    Top right.

    :::

    ::: figure

    ![](bottom-right.png)

    Bottom right.

    :::

Tall panel on the left with two stacked panels on the right.

:::
```

## Layout map: wide panel spanning columns

```smd
::: figure [a a | b c]

    ::: figure

    ![](wide-top.png)

    Spans both columns.

    :::

    ::: figure

    ![](bottom-left.png)

    Bottom left.

    :::

    ::: figure

    ![](bottom-right.png)

    Bottom right.

    :::

Wide panel on top with two panels below.

:::
```

## Layout map with empty cell

```smd
::: figure [a . | b c]

    ::: figure

    ![](top-left.png)

    Top left.

    :::

    ::: figure

    ![](bottom-left.png)

    Bottom left.

    :::

    ::: figure

    ![](bottom-right.png)

    Bottom right.

    :::

Top-right cell intentionally empty.

:::
```

## Layout map with custom column widths

```smd
::: figure [30 70 : a b | a c]

    ::: figure

    ![](narrow-tall.png)

    Narrow left, spans both rows.

    :::

    ::: figure

    ![](wide-top.png)

    Wide right, top.

    :::

    ::: figure

    ![](wide-bottom.png)

    Wide right, bottom.

    :::

30:70 split with left panel spanning both rows.

:::
```

## Columns with gap

```smd
::: figure [40 g20 40]

    ::: figure

    ![](left.png)

    Left.

    :::

    ::: figure

    ![](right.png)

    Right.

    :::

Two columns with an explicit gap.

:::
```

## Padding for below-image annotations

Use bottom padding so overlays (scale bars, labels) sit below the image:

````smd
::: figure {pad="0 0 56 0"}

![](image.png)

```svg overlay
<svg viewBox="0 0 600 356" xmlns:s="https://stencila.io/svg">
  <s:scale-bar at="#s:bottom-left" dx="40" dy="-26" length="130" label="20 μm"/>
  <s:compass at="#s:bottom-right" dx="-40" dy="-26" size="36"/>
</svg>
```

Image with scale bar and compass below.

:::
````

The `viewBox` height = image height (300) + bottom padding (56) = 356.
