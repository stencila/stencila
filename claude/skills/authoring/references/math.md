<!-- Generated from site/docs/documents by claude/scripts/gen-references.sh. Do not edit. -->


# Math

Stencila Markdown supports inline and block mathematics. You can author math using TeX-style syntax, AsciiMath, or MathML, depending on what is most convenient for your workflow.

# Inline math

Inline math is used when the expression should appear inside a paragraph.

## TeX-style inline math

Use inline code with a math language tag for inline math:

```smd
A TeX equation `2 \pi r^2`{tex}.
```

## AsciiMath inline math

```smd
An AsciiMath equation `2 pi r^2`{asciimath}.
```

## MathML inline math

```smd
A MathML equation `<math><mrow><mn>2</mn><mi>π</mi><msup><mi>r</mi><mn>2</mn></msup></mrow></math>`{mathml}.
```

# Block math

Block math is used when the expression should stand on its own line.

## TeX block math

When no language is specified, Stencila treats `$$ ... $$` math blocks as TeX:

```smd
$$
2 \pi r^2
$$
```

## AsciiMath block math

````smd
```asciimath
2 pi r^2
```
````

## MathML block math

````smd
```mathml
<math display="block"><mrow><mn>2</mn><mi>π</mi><msup><mi>r</mi><mn>2</mn></msup></mrow></math>
```
````

# Choosing a math format

- Use **TeX** if you already write mathematics in LaTeX-style syntax.
- Use **AsciiMath** if you prefer a lighter plain-text syntax.
- Use **MathML** if you need an explicit XML representation of the math structure.

For most authoring workflows, TeX-style math is likely to be the most familiar option.

