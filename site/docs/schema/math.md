---
title: Math
description: An abstract base type for mathematical content.
---

This is an abstract base type used in Stencila Schema for mathematical content.

It exists to provide a shared representation for inline and block math,
including source math code, math language, authorship, provenance, and
compilation-related metadata such as MathML and diagnostics. This keeps
mathematical content aligned across authoring, rendering, and interchange
workflows.

Key properties include `code`, `mathLanguage`, `compilationDigest`,
`compilationMessages`, `mathml`, and `images`.


# Analogues

The following external types, elements, or nodes are similar to a `Math`:

- [MathML](https://www.w3.org/Math/): Close interchange analogue for mathematical notation, though Stencila stores source math plus compiled MathML and related metadata.
- Pandoc [`Math`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Math): Close Pandoc analogue for shared mathematical content across inline and display contexts.

# Properties

The `Math` type has these properties:

| Name                  | Description                                                         | Type                                              | Inherited from          |
| --------------------- | ------------------------------------------------------------------- | ------------------------------------------------- | ----------------------- |
| `code`                | The code of the equation in the `mathLanguage`.                     | [`Cord`](./cord.md)                               | -                       |
| `mathLanguage`        | The language used for the equation e.g tex, mathml, asciimath.      | [`String`](./string.md)                           | -                       |
| `authors`             | The authors of the math.                                            | [`Author`](./author.md)*                          | -                       |
| `provenance`          | A summary of the provenance of the math.                            | [`ProvenanceCount`](./provenance-count.md)*       | -                       |
| `compilationDigest`   | A digest of the `code` and `mathLanguage`.                          | [`CompilationDigest`](./compilation-digest.md)    | -                       |
| `compilationMessages` | Messages generated while parsing and compiling the math expression. | [`CompilationMessage`](./compilation-message.md)* | -                       |
| `mathml`              | The MathML transpiled from the `code`.                              | [`String`](./string.md)                           | -                       |
| `images`              | Images of the math.                                                 | [`ImageObject`](./image-object.md)*               | -                       |
| `id`                  | The identifier for this item.                                       | [`String`](./string.md)                           | [`Entity`](./entity.md) |

# Related

The `Math` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: [`MathBlock`](./math-block.md), [`MathInline`](./math-inline.md)

# Bindings

The `Math` type is represented in:

- [JSON-LD](https://stencila.org/Math.jsonld)
- [JSON Schema](https://stencila.org/Math.schema.json)
- Python class [`Math`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Math`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/math.rs)
- TypeScript class [`Math`](https://github.com/stencila/stencila/blob/main/ts/src/types/Math.ts)

***

This documentation was generated from [`Math.yaml`](https://github.com/stencila/stencila/blob/main/schema/Math.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
