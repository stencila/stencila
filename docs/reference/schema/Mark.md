# Mark

**A base class for nodes that mark some other inline content in some way (e.g. as being emphasised, or quoted).**

## Properties

| Name        | `@id`                                                       | Type                                       | Description                         | Inherited from      |
| ----------- | ----------------------------------------------------------- | ------------------------------------------ | ----------------------------------- | ------------------- |
| **content** | [stencila:content](https://schema.stenci.la/content.jsonld) | Array of [InlineContent](InlineContent.md) | The content that is marked.         | [Mark](Mark.md)     |
| id          | [schema:id](https://schema.org/id)                          | string                                     | The identifier for this item.       | [Entity](Entity.md) |
| meta        | [stencila:meta](https://schema.stenci.la/meta.jsonld)       | object                                     | Metadata associated with this item. | [Entity](Entity.md) |

## Related

- Parent: [Entity](Entity.md)
- Descendants: [Delete](Delete.md), [Emphasis](Emphasis.md), [NontextualAnnotation](NontextualAnnotation.md), [Quote](Quote.md), [Strong](Strong.md), [Subscript](Subscript.md), [Superscript](Superscript.md)

## Available as

- [JSON-LD](https://schema.stenci.la/Mark.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Mark.schema.json)
- Python [`class Mark`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Mark)
- TypeScript [`interface Mark`](https://stencila.github.io/schema/ts/docs/interfaces/mark.html)
- R [`class Mark`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Mark`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Mark.html)

## Source

This documentation was generated from [Mark.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/Mark.schema.yaml).
