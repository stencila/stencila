---
title: Math
description: Abstract base type for a mathematical variable or equation.
config:
  publish:
    ghost:
      type: post
      slug: math
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Math
---

This is a base type for `MathFragment` and `MathBlock` and should not
normally be instantiated.
This type has a similar structure and purpose to `CodeStatic` which is a base type
for `CodeFragment`, `CodeBlock` etc.


# Properties

The `Math` type has these properties:

| Name                  | Description                                                         | Type                                                                                         | Inherited from                                                     | `JSON-LD @id`                                | Aliases                                                                                                            |
| --------------------- | ------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)         | -                                                                                                                  |
| `code`                | The code of the equation in the `mathLanguage`.                     | [`Cord`](https://stencila.ghost.io/docs/reference/schema/cord)                               | -                                                                  | `stencila:code`                              | -                                                                                                                  |
| `mathLanguage`        | The language used for the equation e.g tex, mathml, asciimath.      | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | -                                                                  | `stencila:mathLanguage`                      | `math-language`, `math_language`                                                                                   |
| `authors`             | The authors of the math.                                            | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                          | -                                                                  | [`schema:author`](https://schema.org/author) | `author`                                                                                                           |
| `provenance`          | A summary of the provenance of the math.                            | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)*       | -                                                                  | `stencila:provenance`                        | -                                                                                                                  |
| `compilationDigest`   | A digest of the `code` and `mathLanguage`.                          | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)    | -                                                                  | `stencila:compilationDigest`                 | `compilation-digest`, `compilation_digest`                                                                         |
| `compilationMessages` | Messages generated while parsing and compiling the math expression. | [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message)* | -                                                                  | `stencila:compilationMessages`               | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |
| `mathml`              | The MathML transpiled from the `code`.                              | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | -                                                                  | `stencila:mathml`                            | -                                                                                                                  |
| `images`              | Images of the math.                                                 | [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)*               | -                                                                  | [`schema:image`](https://schema.org/image)   | `image`                                                                                                            |

# Related

The `Math` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: [`MathBlock`](https://stencila.ghost.io/docs/reference/schema/math-block), [`MathInline`](https://stencila.ghost.io/docs/reference/schema/math-inline)

# Bindings

The `Math` type is represented in:

- [JSON-LD](https://stencila.org/Math.jsonld)
- [JSON Schema](https://stencila.org/Math.schema.json)
- Python class [`Math`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/math.py)
- Rust struct [`Math`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/math.rs)
- TypeScript class [`Math`](https://github.com/stencila/stencila/blob/main/ts/src/types/Math.ts)

# Source

This documentation was generated from [`Math.yaml`](https://github.com/stencila/stencila/blob/main/schema/Math.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
