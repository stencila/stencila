# Plot token review reference

Use this file when reviewing plot theming or any claim that a document theme will also style Python or R plots.

For a comprehensive current list of builtin plot tokens, use the CLI:

```sh
stencila themes tokens --scope plot
```

Use machine-readable output when you need to inspect or post-process the inventory:

```sh
stencila themes tokens --scope plot --as json
```

## Core rule

Only `--plot-*` tokens transfer to Python and R kernels. General document tokens do not theme Python or R plots unless the corresponding plot tokens are also set.

## Theme selection

Use the CLI inventory to confirm exact values and related tokens around:

- `--plot-theme`
  - `custom` applies the plot token system (default)
  - `none` disables plot theming for web, Python, and R
  - named presets are merged before custom overrides are extracted

Available plot presets:

- `bold` — stronger visual emphasis with thicker lines and bolder grid
- `light` — lighter, more minimal plot styling

Set `--plot-theme` to a preset name to load that preset's CSS, then layer `--plot-*` overrides on top. The preset is inserted before the theme's `:root` block, so theme-level tokens always win.

## Major plot token families to verify

Use `stencila themes tokens --scope plot` to inspect exact names in families such as:

- dimensions
- categorical palette
- sequential ramp
- typography and backgrounds
- axes and grid
- legends
- tooltips
- marks

## How to use the CLI and this reference together

1. Use `stencila themes tokens --scope plot` to get the current builtin token inventory.
2. Verify exact plot token names from the CLI output against what the artifact uses.
3. Use this reference to understand the important constraint that only `--plot-*` tokens transfer to Python and R.
4. Flag any artifact that claims plot theming without `--plot-*` tokens.

## Cross-target cautions

- Plot tokens should be set at document-root `:root` for reliable extraction.
- Inline overrides inside narrower blocks may not affect JS plot renderers that read from the document root.
- Some tokens do not map equally across browser, Python, and R outputs.
- `--plot-subtitle-size` is an example of a token that is not currently mapped in matplotlib.
- Several interaction-oriented plot tokens are not applied in R.
- Describe cross-renderer parity as approximate unless the relevant renderers have been checked.

## Dark-mode plot review

Plot tokens may have `*-dark` variants (e.g., `--plot-background-dark`). When reviewing:

- Check whether plot colors are suitable for both light and dark backgrounds.
- Flag missing dark variants for plot background, text, and grid tokens when the theme targets web outputs.

## Validation workflow

When plot theming matters, recommend checks such as:

1. inspect a web-rendered plot
2. execute the Python or R pathway that consumes the theme
3. compare palette, typography, grid, and title sizing across renderers
4. confirm whether a token is actually mapped before approving renderer-specific claims
5. run `stencila themes validate theme.css` after editing the file; use `--strict` if unknown tokens should fail
