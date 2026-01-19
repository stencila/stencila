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

| Name                  | Description                                                         | Type                                              | Inherited from          | `JSON-LD @id`                                | Aliases                                                                                                            |
| --------------------- | ------------------------------------------------------------------- | ------------------------------------------------- | ----------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                                       | [`String`](./string.md)                           | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)         | -                                                                                                                  |
| `code`                | The code of the equation in the `mathLanguage`.                     | [`Cord`](./cord.md)                               | -                       | `stencila:code`                              | -                                                                                                                  |
| `mathLanguage`        | The language used for the equation e.g tex, mathml, asciimath.      | [`String`](./string.md)                           | -                       | `stencila:mathLanguage`                      | `math-language`, `math_language`                                                                                   |
| `authors`             | The authors of the math.                                            | [`Author`](./author.md)*                          | -                       | [`schema:author`](https://schema.org/author) | `author`                                                                                                           |
| `provenance`          | A summary of the provenance of the math.                            | [`ProvenanceCount`](./provenance-count.md)*       | -                       | `stencila:provenance`                        | -                                                                                                                  |
| `compilationDigest`   | A digest of the `code` and `mathLanguage`.                          | [`CompilationDigest`](./compilation-digest.md)    | -                       | `stencila:compilationDigest`                 | `compilation-digest`, `compilation_digest`                                                                         |
| `compilationMessages` | Messages generated while parsing and compiling the math expression. | [`CompilationMessage`](./compilation-message.md)* | -                       | `stencila:compilationMessages`               | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |
| `mathml`              | The MathML transpiled from the `code`.                              | [`String`](./string.md)                           | -                       | `stencila:mathml`                            | -                                                                                                                  |
| `images`              | Images of the math.                                                 | [`ImageObject`](./image-object.md)*               | -                       | [`schema:image`](https://schema.org/image)   | `image`                                                                                                            |

# Related

The `Math` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: [`MathBlock`](./math-block.md), [`MathInline`](./math-inline.md)

# Bindings

The `Math` type is represented in:

- [JSON-LD](https://stencila.org/Math.jsonld)
- [JSON Schema](https://stencila.org/Math.schema.json)
- Python class [`Math`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/math.py)
- Rust struct [`Math`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/math.rs)
- TypeScript class [`Math`](https://github.com/stencila/stencila/blob/main/ts/src/types/Math.ts)

# Source

This documentation was generated from [`Math.yaml`](https://github.com/stencila/stencila/blob/main/schema/Math.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
