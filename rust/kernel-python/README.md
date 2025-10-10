# `stencila-kernel-python`

## Theming tests

The `test-themes.py` script is for testing the application of Stencila theme variables to matplotlib plots. It creates the `test-themes` directory containing `png` files generated with different themes.

To regenerate the plots:

```bash
make -C rust/kernel-python/ test-themes
```

### Current themes

There are currently three contrasting themes, `monochrome`, `vintage`, and `cyberpunk`, aimed at testing the application of all theme variables.

### Current plots

#### Matplotlib Plots

1. Scatter plot - Tests point size, colors, axis labels, title
2. Line plot - Tests line width, colors, legend
3. Bar plot - Tests color palette, axis titles
4. Histogram - Tests background, fill color
5. Box plot - Tests color palette, axis formatting
6. Multiple panels - Tests multi-panel layout, consistent theming
7. Annotated plot - Tests title, subtitle (via suptitle), caption styling

### Tested Theme Variables

The tests verify that the following Stencila theme variables are correctly applied:

#### Currently Mapped Variables

- `plot-background` - Plot background color
- `plot-axis-line-color` - Axis line color
- `plot-axis-line-width` - Axis line width
- `plot-axis-title-color` - Axis title color
- `plot-axis-title-size` - Axis title size
- `plot-color-1` through `plot-color-12` - Color palette
- `plot-font-family` - Font family
- `plot-font-size` - Base font size
- `plot-grid-color` - Grid line color
- `plot-grid-width` - Grid line width
- `plot-legend-background` - Legend background
- `plot-legend-border-color` - Legend border color
- `plot-legend-border-width` - Legend border width
- `plot-legend-text-color` - Legend text color
- `plot-legend-size` - Legend text size
- `plot-line-cap` - Line end style
- `plot-line-join` - Line join style
- `plot-line-width` - Line width
- `plot-point-size` - Point size
- `plot-text-color` - Main text color
- `plot-tick-color` - Tick mark color
- `plot-tick-size` - Tick mark size
- `plot-tick-width` - Tick mark width
- `plot-title-size` - Title size

#### Documented But Not Yet Mapped

See comments in `/rust/kernel-python/src/theme.py` for complete list of unmapped parameters (100+ documented)
