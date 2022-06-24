# Code Block

**A code block.**

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

- [JSON-LD](https://schema.stenci.la/CodeBlock.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/CodeBlock.schema.json)
- Python [`class CodeBlock`](https://stencila.github.io/schema/python/docs/types.html#schema.types.CodeBlock)
- TypeScript [`interface CodeBlock`](https://stencila.github.io/schema/ts/docs/interfaces/codeblock.html)
- R [`class CodeBlock`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct CodeBlock`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.CodeBlock.html)

## Source

This documentation was generated from [CodeBlock.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/CodeBlock.schema.yaml).
