---
title: Math
description: Abstract base type for a mathematical variable or equation.
---

This is a base type for `MathFragment` and `MathBlock` and should not
normally be instantiated.
This type has a similar structure and purpose to `CodeStatic` which is a base type
for `CodeFragment`, `CodeBlock` etc.


# Properties

The `Math` type has these properties:

| Name                  | Description                                                         | Type                                              | Inherited from          |
| --------------------- | ------------------------------------------------------------------- | ------------------------------------------------- | ----------------------- |
| `id`                  | The identifier for this item.                                       | [`String`](./string.md)                           | [`Entity`](./entity.md) |
| `code`                | The code of the equation in the `mathLanguage`.                     | [`Cord`](./cord.md)                               | -                       |
| `mathLanguage`        | The language used for the equation e.g tex, mathml, asciimath.      | [`String`](./string.md)                           | -                       |
| `authors`             | The authors of the math.                                            | [`Author`](./author.md)*                          | -                       |
| `provenance`          | A summary of the provenance of the math.                            | [`ProvenanceCount`](./provenance-count.md)*       | -                       |
| `compilationDigest`   | A digest of the `code` and `mathLanguage`.                          | [`CompilationDigest`](./compilation-digest.md)    | -                       |
| `compilationMessages` | Messages generated while parsing and compiling the math expression. | [`CompilationMessage`](./compilation-message.md)* | -                       |
| `mathml`              | The MathML transpiled from the `code`.                              | [`String`](./string.md)                           | -                       |
| `images`              | Images of the math.                                                 | [`ImageObject`](./image-object.md)*               | -                       |

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
