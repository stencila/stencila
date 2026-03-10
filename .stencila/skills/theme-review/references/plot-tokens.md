# Plot token review reference

Use this file when reviewing plot theming or any claim that a document theme will also style Python or R plots.

## Core rule

Only `--plot-*` tokens transfer to Python and R kernels. General document tokens do not theme Python or R plots unless the corresponding plot tokens are also set.

## What to verify

Use `stencila themes tokens --scope plot` to inspect exact names for plot token groups such as:

- theme selection
- dimensions
- categorical palette
- sequential ramp
- typography and backgrounds
- axes and grid
- legends
- tooltips
- marks

## Review cautions

- Plot tokens should be set at document-root `:root` for reliable extraction.
- Inline overrides inside narrower blocks may not affect JS plot renderers that read from the document root.
- Some tokens do not map equally across browser, Python, and R outputs.
- `--plot-subtitle-size` is an example of a token that is not currently mapped in matplotlib.
- Describe cross-renderer parity as approximate unless the relevant renderers have been checked.
