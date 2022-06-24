# Emphasis

**Emphasised content.**

Analogues of `Delete` in other schema include: - HTML [`<em>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em) - JATS XML [`<italic>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/italic.html) - MDAST [`Emphasis`](https://github.com/syntax-tree/mdast#emphasis) - Pandoc [`Emph`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L256)

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
    "The following content has extra ",
    {
      "type": "Emphasis",
      "content": ["emphasis"]
    }
  ]
}
```

## Related

- Parent: [Mark](Mark.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Emphasis.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Emphasis.schema.json)
- Python [`class Emphasis`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Emphasis)
- TypeScript [`interface Emphasis`](https://stencila.github.io/schema/ts/docs/interfaces/emphasis.html)
- R [`class Emphasis`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Emphasis`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Emphasis.html)

## Source

This documentation was generated from [Emphasis.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/Emphasis.schema.yaml).
