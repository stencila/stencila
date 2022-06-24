# Quote

**Inline, quoted content.**

## Properties

| Name        | `@id`                                                       | Type                                       | Description                         | Inherited from      |
| ----------- | ----------------------------------------------------------- | ------------------------------------------ | ----------------------------------- | ------------------- |
| **content** | [stencila:content](https://schema.stenci.la/content.jsonld) | Array of [InlineContent](InlineContent.md) | The content that is marked.         | [Mark](Mark.md)     |
| cite        | [stencila:cite](https://schema.stenci.la/cite.jsonld)       | [Cite](Cite.md) _or_ Format 'uri'          | The source of the quote.            | [Quote](Quote.md)   |
| id          | [schema:id](https://schema.org/id)                          | string                                     | The identifier for this item.       | [Entity](Entity.md) |
| meta        | [stencila:meta](https://schema.stenci.la/meta.jsonld)       | object                                     | Metadata associated with this item. | [Entity](Entity.md) |

## Related

- Parent: [Mark](Mark.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Quote.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Quote.schema.json)
- Python [`class Quote`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Quote)
- TypeScript [`interface Quote`](https://stencila.github.io/schema/ts/docs/interfaces/quote.html)
- R [`class Quote`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Quote`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Quote.html)

## Source

This documentation was generated from [Quote.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/Quote.schema.yaml).
