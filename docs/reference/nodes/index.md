Index of nodes

# Parameter

Use a [`Parameter`](parameter.md) node to specify a parameter of the document e.g.

&[my_par]

# For

::: for item in [1,2,3]

:::

# If

::: if true

:::

# Include

/includee.md

# Call

/callee.md()

# Math Fragment

Use a [`MathFragment`](math-fragment.md) for inline math in TeX or AsciiMath.

$\pi r^2$

# Math Block

Use a [`MathBlock`](math-block.md) for block, a.k.a "display", math in TeX or AsciiMath.

$$
\pi r^2
$$

# Span

Use a [`Span`](span.md) to style inline text e.g.

This [text is styled]`rounded px-1 bg-rose-100 text-rose-600` using a [`Span`](span.md)

# Division

Use a [`Division`](division.md) to style blocks of content e.g.

::: rounded p-1 bg-green-100 border-green-200 text(green-600 center)

This paragraph is styled using a [`Division`](division.md)!

:::
