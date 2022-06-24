# Delete

**Content that is marked for deletion**

Analogues of `Delete` in other schema include: - HTML [`<del>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del) - JATS XML [`<strike>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/strike.html) - MDAST [`Delete`](https://github.com/syntax-tree/mdast#delete) - Pandoc [`Strikeout`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L258)

## Properties

| Name        | `@id`                                                       | Type                                       | Description                         | Inherited from      |
| ----------- | ----------------------------------------------------------- | ------------------------------------------ | ----------------------------------- | ------------------- |
| **content** | [stencila:content](https://schema.stenci.la/content.jsonld) | Array of [InlineContent](InlineContent.md) | The content that is marked.         | [Mark](Mark.md)     |
| id          | [schema:id](https://schema.org/id)                          | string                                     | The identifier for this item.       | [Entity](Entity.md) |
| meta        | [stencila:meta](https://schema.stenci.la/meta.jsonld)       | object                                     | Metadata associated with this item. | [Entity](Entity.md) |

## Examples

```json
{
  "type": "Paragraph",
  "content": [
    "The following is ",
    {
      "type": "Delete",
      "content": ["marked for deletion"]
    },
    "."
  ]
}
```

## Related

- Parent: [Mark](Mark.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Delete.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Delete.schema.json)
- Python [`class Delete`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Delete)
- TypeScript [`interface Delete`](https://stencila.github.io/schema/ts/docs/interfaces/delete.html)
- R [`class Delete`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Delete`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Delete.html)

## Source

This documentation was generated from [Delete.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/Delete.schema.yaml).
