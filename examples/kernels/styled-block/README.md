# `StyledBlock` compilation examples

When `StyledInline` nodes are compiled the `code` is transpiled into the `css` and `classList` properties.

## `static.md`

Has several `StyledBlock` nodes, one of which has a warning. Useful for testing that styling, or any warnings, are displayed correctly.

## `dynamic.md`

An example of using variables defined in a variety of other kernels to style a paragraph using those variables. Tests that variables can be requested by the `StyleKernel` instance from other kernel instances.
