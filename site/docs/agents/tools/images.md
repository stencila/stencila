---
title: Image Tools
description: Tools for inspecting and working with images in Stencila agent workflows.
---

## `inspect_image`

A read-only tool for determining reliable coordinates for points and regions of interest in images. It overlays labeled grids, zooms into crop regions, and places probe crosshair markers — returning an annotated PNG and structured JSON metadata with coordinates in the active coordinate space.

Use `inspect_image` to verify how features map to coordinates before authoring SVG overlay annotations. Start with a grid overlay to get the lay of the land, crop to zoom into regions of interest, then place probes to verify candidate coordinates.

### Parameters

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `file_path` | string | yes | Path to the source raster image (PNG, JPEG, GIF, TIFF) |
| `coordinate_space` | object | | Custom coordinate space. Omit for raw image pixel coordinates. See [coordinate space](#coordinate-space) |
| `grid` | object | | Grid overlay configuration. See [grid overlay](#grid-overlay) |
| `crop` | object | | Rectangular crop region in active coordinate space. See [crop](#crop) |
| `probes` | array | | Probe markers at candidate coordinates. See [probes](#probes) |
| `theme` | string | | Visual theme for grid/probe rendering: `"auto"`, `"light"`, `"dark"`. Default: `"auto"` |
| `sample_pixels` | boolean | | Include sampled pixel color for each in-image probe. Default: false |

### Coordinate space

The `coordinate_space` parameter maps the image into a custom coordinate system. When omitted, the tool operates in raw image pixel coordinates with origin `(0, 0)` at the top-left pixel.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `viewbox` | object | | Maps the image into a custom coordinate system (matching Stencila SVG overlay viewBox). Has `x`, `y`, `width`, `height` fields (all required, `width` and `height` must be positive) |
| `pad` | object | | Padding extending the coordinate space beyond image bounds. Has `top`, `right`, `bottom`, `left` fields (all required, all non-negative). Follows CSS order matching Stencila's figure `padding` attribute |

When `pad` is specified without `viewbox`, the viewbox is auto-computed: the canvas becomes `(image_width + pad.left + pad.right) x (image_height + pad.top + pad.bottom)` and the image is placed at offset `(pad.left, pad.top)`.

When both `viewbox` and `pad` are specified, the image occupies the sub-region of the viewbox after padding is subtracted: `image_width_in_active = viewbox.width - pad.left - pad.right`.

### Grid overlay

The `grid` parameter overlays labeled lines on the image so that feature locations can be estimated by reading axis labels. Provide divisions OR spacing, not both.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `x_divisions` | integer | | Number of divisions along the x-axis (minimum: 1). Mutually exclusive with `spacing` |
| `y_divisions` | integer | | Number of divisions along the y-axis (minimum: 1). Mutually exclusive with `spacing` |
| `spacing` | number | | Distance between grid lines in active-coordinate units. Mutually exclusive with `x_divisions`/`y_divisions` |
| `show_labels` | boolean | | Whether to show coordinate labels. Default: true |

**Division-based grids** divide each axis into N equal parts. For example, `x_divisions: 10` on a 500-unit-wide canvas places lines at 0, 50, 100, ..., 500.

**Spacing-based grids** place lines at fixed intervals starting from the origin of the active coordinate space. For example, `spacing: 50` on a 500-unit-wide canvas places vertical lines at 0, 50, 100, ..., 500.

### Crop

The `crop` parameter extracts and enlarges a rectangular region of interest for closer inspection. Coordinate labels continue to represent the active coordinate space, not crop-local coordinates.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `x` | number | yes | Top-left x in active coordinate space |
| `y` | number | yes | Top-left y in active coordinate space |
| `width` | number | yes | Region width (must be > 0) |
| `height` | number | yes | Region height (must be > 0) |
| `zoom` | number | | Enlargement factor. Default: auto-fit to ~1024px on the longest side |

Crop regions may extend into padding areas when a padded coordinate space is active — the padding portion is rendered as a neutral background.

### Probes

The `probes` parameter places visible crosshair markers at candidate coordinates. Each probe is rendered as a numbered crosshair with a small gap at center so the underlying pixel remains visible.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `x` | number | yes | x in active coordinate space |
| `y` | number | yes | y in active coordinate space |
| `id` | string | | Probe identifier (default: 1-based index) |
| `label` | string | | Text label displayed near the probe |
| `color` | string | | Marker color, e.g. `"#ff0000"` (default: auto-contrast) |

Probes outside the inspectable canvas are reported in metadata with `"in_canvas": false` but are not rendered visually.

### Output

Every call returns an annotated PNG image and a JSON metadata object with these fields:

- **`dimensions`** — source image pixel dimensions (`width`, `height`). Always present.
- **`coordinate_space`** — the active coordinate space: `type` (`"image"` or `"viewbox"`), `origin`, `size`, and when using a viewbox, `pad` and `image_region`. Always present.
- **`grid`** — grid configuration as applied: division or spacing values plus `x_values` and `y_values` arrays. Present when grid mode is active.
- **`crop`** — crop region and zoom factor. Present when crop mode is active.
- **`probes`** — per-probe results: `id`, `x`, `y`, `in_image`, `in_canvas`, and optionally `pixel` (hex color when `sample_pixels` is true and `in_image` is true). Present when probes are provided.

### Mode composition

Grid, crop, and probe modes compose freely in a single call:

- **crop + grid** — zoom into a region and read precise coordinates from grid labels
- **crop + probe** — verify fine placement within a zoomed region
- **grid + probe** — coarse verification on the full image
- **crop + grid + probe** — all three together for the most detailed inspection

### Typical workflow

1. Grid — get the lay of the land

    ```
    inspect_image(file_path: "figure.png", grid: {x_divisions: 10, y_divisions: 10})
    ```

2. Crop + grid — zoom into the region of interest

    ```
    inspect_image(
      file_path: "figure.png",
      crop: {x: 80, y: 100, width: 140, height: 100},
      grid: {x_divisions: 5, y_divisions: 5}
    )
    ```

3. Probe — verify candidate coordinates

    ```
    inspect_image(
      file_path: "figure.png",
      probes: [{id: "tip", x: 112, y: 160}, {id: "base", x: 185, y: 118}]
    )
    ```

4. Author the SVG overlay using validated coordinates

5. `snap` — verify the final rendered result once

Steps 1–3 replace guessing coordinates, rendering the full document with `snap`, seeing the result is wrong, re-guessing, and re-rendering.

### ViewBox workflow

For figures with padding (e.g. right padding for callout labels):

1. Grid with coordinate space — labels match the overlay's viewBox

    ```
    inspect_image(
      file_path: "figure.png",
      coordinate_space: {pad: {top: 0, right: 220, bottom: 0, left: 0}},
      grid: {x_divisions: 10, y_divisions: 10}
    )
    ```

2. Probe in padding area — verify a callout label lands in the right spot

    ```
    inspect_image(
      file_path: "figure.png",
      coordinate_space: {pad: {top: 0, right: 220, bottom: 0, left: 0}},
      probes: [{id: "label", x: 560, y: 80}]
    )
    ```

### Pixel sampling

When `sample_pixels: true`, each probe that falls within the source image (`in_image: true`) includes a `pixel` field with the sampled color as a `"#rrggbb"` hex string. Probes in padding areas or outside the canvas do not have a `pixel` field.

```
inspect_image(
  file_path: "figure.png",
  probes: [{id: "bg", x: 10, y: 10}, {id: "feature", x: 250, y: 375}],
  sample_pixels: true
)
```

### Practical tips

- **Grid-first workflow**: start with a coarse grid to identify regions, then crop + finer grid to read precise coordinates, then probes to verify before authoring markup.
- **Use coordinate space for SVG overlays**: when annotating figures with Stencila's SVG overlay system, pass the same `viewbox` and `pad` values so grid labels and probe coordinates match the overlay coordinate system directly.
- **Auto theme handles most cases**: the `"auto"` theme analyzes image luminance and chooses contrasting grid/label colors. Use `"light"` or `"dark"` only if auto-detection picks poorly for a specific image.
- **Crop regions preserve source coordinates**: labels on cropped output show active-coordinate-space values, not crop-local values — coordinates read from a cropped view can be used directly in overlay markup.
- **Probes are cheaper than snap**: verifying coordinates with `inspect_image` probes is much faster than a full `snap` round-trip through the browser. Reserve `snap` for final rendered verification.
- **Spacing-based grids for regular intervals**: use `spacing` when you want grid lines at round-number intervals (e.g. every 50 units). Use `x_divisions`/`y_divisions` when you want a fixed number of divisions regardless of canvas size.
- **Supported formats**: PNG, JPEG, GIF (first frame only), and TIFF. SVG source images are not supported.
