# Math

**A mathematical variable or equation.**

This is a base type for `MathFragment` and `MathBlock` and should not normally be instantiated. This type has a similar structure and purpose to `Code` which is a base type for `CodeFragment`, `CodeBlock` etc.

## Properties

| Name         | `@id`                                                                 | Type            | Description                                                                | Inherited from      |
| ------------ | --------------------------------------------------------------------- | --------------- | -------------------------------------------------------------------------- | ------------------- |
| **text**     | [schema:text](https://schema.org/text)                                | string          | The text of the equation in the language.                                  | [Math](Math.md)     |
| errors       | [stencila:errors](https://schema.stenci.la/errors.jsonld)             | Array of string | Errors that occurred when parsing the math equation. See note [1](#notes). | [Math](Math.md)     |
| id           | [schema:id](https://schema.org/id)                                    | string          | The identifier for this item.                                              | [Entity](Entity.md) |
| mathLanguage | [stencila:mathLanguage](https://schema.stenci.la/mathLanguage.jsonld) | string          | The language used for the equation e.g tex, mathml, asciimath.             | [Math](Math.md)     |
| meta         | [stencila:meta](https://schema.stenci.la/meta.jsonld)                 | object          | Metadata associated with this item.                                        | [Entity](Entity.md) |

## Notes

1. **errors** : This property is an array of strings. Compare this to `CodeChunk.errors` which is an array of `CodeError` nodes. Strings are considered to be sufficient for math parsing errors which usually won't have stack traces, line numbers etc.

## Related

- Parent: [Entity](Entity.md)
- Descendants: [MathBlock](MathBlock.md), [MathFragment](MathFragment.md)

## Available as

- [JSON-LD](https://schema.stenci.la/Math.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Math.schema.json)
- Python [`class Math`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Math)
- TypeScript [`interface Math`](https://stencila.github.io/schema/ts/docs/interfaces/math.html)
- R [`class Math`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Math`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Math.html)

## Source

This documentation was generated from [Math.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/Math.schema.yaml).
