# Plot token reference

Use this file for plot-theming workflow guidance, major plot token families, and cross-target caveats.

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
  - `custom` applies the plot token system
  - `none` disables plot theming
  - named presets can be merged before custom overrides are extracted

## Major plot token families

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
2. Choose exact plot token names from the CLI output.
3. Use this reference to understand the important constraint that only `--plot-*` tokens transfer to Python and R.
4. Validate in each renderer that matters because some tokens do not map equally across browser, Python, and R outputs.

## Cross-target cautions

- Set plot tokens on document-root `:root` for consistent extraction.
- Inline overrides inside other blocks may not affect JS plot renderers that read from the document root.
- Some tokens do not map equally across targets; for example, `--plot-subtitle-size` is not currently mapped in matplotlib.
- Describe cross-renderer parity as approximate unless the user has validated each target.

## Validation workflow

When plot theming matters, recommend checks such as:

1. inspect a web-rendered plot
2. execute the Python or R pathway that consumes the theme
3. compare palette, typography, grid, and title sizing across renderers
4. confirm whether a token is actually mapped before promising renderer-specific behavior
5. run `stencila themes validate theme.css` after editing the file; use `--strict` if unknown tokens should fail

## Verified source basis

Localized from the Stencila plots token documentation when this skill was prepared.
