Welcome to the Stencila's reference documentation for node types. This document serves as an overview and index to the reference documentation for each node type.

# Code blocks

Use a [`CodeBlock`](code-block.md) for static non-executable blocks of code e.g.

```javascript
// Some JavaScript code
2 * Math.PI
```

# Code fragments

Use a [`CodeFragment`](code-fragment.md) for static non-executable fragments of inline code e.g.

Some R code `2 * pi`{r} in a paragraph.

# Code chunks

Use a [`CodeChunk`](code-chunk.md) for executable blocks of code e.g.

```python exec
import sys

f"Hello from Python {sys.version_info.major}.{sys.version_info.minor}"
```

# Code expressions

Use a [`CodeExpression`](code-expression.md) for executable fragments of inline code producing an output e.g.

An R expression `6 * 7`{r exec} in a paragraph.

# Parameter

Use a [`Parameter`](parameter.md) to specify a value in the document which can be changed by users and used in `Call`s of the document e.g.

&[my_par]{num min=1.0 max=10.0 mult=0.1 label="Label"}

# Button

Use a [`Button`](button.md) to trigger

#[Hello world!]

# For

::: for item in [1,2,3,4]

Content to repeat for each item

:::

# If

::: if true

Content to display if true

:::

# Include

/includee.md

# Call

/callee.md()

# Math Block

Use a [`MathBlock`](math-block.md) for block, a.k.a "display", math in TeX or AsciiMath.

$$
\pi r^2
$$

# Math Fragment

Use a [`MathFragment`](math-fragment.md) for inline math in TeX or AsciiMath.

$\pi r^2$

# Span

Use a [`Span`](span.md) to style inline text e.g.

This [text is styled]`rounded px-1 bg-rose-100 text-rose-600` using a [`Span`](span.md)

# Division

Use a [`Division`](division.md) to style blocks of content e.g.

::: rounded p-1 bg-green-100 border-green-200 text(green-600 center)

This paragraph is styled using a [`Division`](division.md)!

:::
