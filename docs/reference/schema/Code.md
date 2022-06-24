# Code

**Base type for non-executable (e.g. `CodeBlock`) and executable (e.g. `CodeExpression`) code nodes.**

The most important, and only required, property of a `Code` node is `text`, a `string` of the source code. There are no restrictions on the length or content of `text` and it is possible for it to be syntactically invalid for the specified `programmingLanguage`.

## Properties

| Name                | `@id`                                                                | Type   | Description                                                                             | Inherited from      |
| ------------------- | -------------------------------------------------------------------- | ------ | --------------------------------------------------------------------------------------- | ------------------- |
| **text**            | [schema:text](https://schema.org/text)                               | string | The text of the code.                                                                   | [Code](Code.md)     |
| id                  | [schema:id](https://schema.org/id)                                   | string | The identifier for this item.                                                           | [Entity](Entity.md) |
| mediaType           | [schema:encodingFormat](https://schema.org/encodingFormat)           | string | Media type, typically expressed using a MIME format, of the code. See note [1](#notes). | [Code](Code.md)     |
| meta                | [stencila:meta](https://schema.stenci.la/meta.jsonld)                | object | Metadata associated with this item.                                                     | [Entity](Entity.md) |
| programmingLanguage | [schema:programmingLanguage](https://schema.org/programmingLanguage) | string | The programming language of the code.                                                   | [Code](Code.md)     |

## Notes

1. **mediaType** : This property allows the differentiation of formats using the same programming language or variants of a programming language. An example is using `programmingLanguage` "json" and `encodingFormat` "application/ld+json" for JSON-LD code examples.

## Related

- Parent: [Entity](Entity.md)
- Descendants: [CodeBlock](CodeBlock.md), [CodeChunk](CodeChunk.md), [CodeExecutable](CodeExecutable.md), [CodeExpression](CodeExpression.md), [CodeFragment](CodeFragment.md)

## Available as

- [JSON-LD](https://schema.stenci.la/Code.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Code.schema.json)
- Python [`class Code`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Code)
- TypeScript [`interface Code`](https://stencila.github.io/schema/ts/docs/interfaces/code.html)
- R [`class Code`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Code`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Code.html)

## Source

This documentation was generated from [Code.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/Code.schema.yaml).
