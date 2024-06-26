# Math

**Abstract base type for a mathematical variable or equation.**

This is a base type for `MathFragment` and `MathBlock` and should not
normally be instantiated.
This type has a similar structure and purpose to `CodeStatic` which is a base type
for `CodeFragment`, `CodeBlock` etc.


**`@id`**: `stencila:Math`

## Properties

The `Math` type has these properties:

| Name                  | Aliases                                                                                                            | `@id`                                        | Type                                                                                                                      | Description                                                         | Inherited from                                                                                   |
| --------------------- | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`                  | -                                                                                                                  | [`schema:id`](https://schema.org/id)         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                           | The identifier for this item.                                       | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `code`                | -                                                                                                                  | `stencila:code`                              | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                               | The code of the equation in the `mathLanguage`.                     | -                                                                                                |
| `mathLanguage`        | `math-language`, `math_language`                                                                                   | `stencila:mathLanguage`                      | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                           | The language used for the equation e.g tex, mathml, asciimath.      | -                                                                                                |
| `authors`             | `author`                                                                                                           | [`schema:author`](https://schema.org/author) | [`Author`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author.md)*                         | The authors of the math.                                            | -                                                                                                |
| `provenance`          | -                                                                                                                  | `stencila:provenance`                        | [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/provenance-count.md)*      | A summary of the provenance of the math.                            | -                                                                                                |
| `compilationDigest`   | `compilation-digest`, `compilation_digest`                                                                         | `stencila:compilationDigest`                 | [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation-digest.md)    | A digest of the `code` and `mathLanguage`.                          | -                                                                                                |
| `compilationMessages` | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` | `stencila:compilationMessages`               | [`CompilationMessage`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/compilation-message.md)* | Messages generated while parsing and compiling the math expression. | -                                                                                                |
| `mathml`              | -                                                                                                                  | `stencila:mathml`                            | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                           | The MathML transpiled from the `code`.                              | -                                                                                                |

## Related

The `Math` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: [`MathBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math-block.md), [`MathInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math-inline.md)

## Bindings

The `Math` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Math.jsonld)
- [JSON Schema](https://stencila.org/Math.schema.json)
- Python class [`Math`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/math.py)
- Rust struct [`Math`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/math.rs)
- TypeScript class [`Math`](https://github.com/stencila/stencila/blob/main/ts/src/types/Math.ts)

## Source

This documentation was generated from [`Math.yaml`](https://github.com/stencila/stencila/blob/main/schema/Math.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
