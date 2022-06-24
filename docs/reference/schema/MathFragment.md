# Math Fragment

**A fragment of math, e.g a variable name, to be treated as inline content.**

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

- Parent: [Math](Math.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/MathFragment.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/MathFragment.schema.json)
- Python [`class MathFragment`](https://stencila.github.io/schema/python/docs/types.html#schema.types.MathFragment)
- TypeScript [`interface MathFragment`](https://stencila.github.io/schema/ts/docs/interfaces/mathfragment.html)
- R [`class MathFragment`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct MathFragment`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.MathFragment.html)

## Source

This documentation was generated from [MathFragment.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/MathFragment.schema.yaml).
