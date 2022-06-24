# Code Fragment

**Inline code.**

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

- Parent: [Code](Code.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/CodeFragment.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/CodeFragment.schema.json)
- Python [`class CodeFragment`](https://stencila.github.io/schema/python/docs/types.html#schema.types.CodeFragment)
- TypeScript [`interface CodeFragment`](https://stencila.github.io/schema/ts/docs/interfaces/codefragment.html)
- R [`class CodeFragment`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct CodeFragment`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.CodeFragment.html)

## Source

This documentation was generated from [CodeFragment.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/CodeFragment.schema.yaml).
