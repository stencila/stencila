# Paragraph

**Paragraph**

Analogues of `Paragraph` in other schema include: - HTML [`<p>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p) - JATS XML [`<p>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/p.html) - MDAST [`Paragraph`](https://github.com/syntax-tree/mdast#Paragraph) - OpenDocument [`<text:p>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415138_253892949) - Pandoc [`Para`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L220)

## Properties

| Name        | `@id`                                                       | Type                                       | Description                         | Inherited from            |
| ----------- | ----------------------------------------------------------- | ------------------------------------------ | ----------------------------------- | ------------------------- |
| **content** | [stencila:content](https://schema.stenci.la/content.jsonld) | Array of [InlineContent](InlineContent.md) | The contents of the paragraph.      | [Paragraph](Paragraph.md) |
| id          | [schema:id](https://schema.org/id)                          | string                                     | The identifier for this item.       | [Entity](Entity.md)       |
| meta        | [stencila:meta](https://schema.stenci.la/meta.jsonld)       | object                                     | Metadata associated with this item. | [Entity](Entity.md)       |

## Examples

```json
{
  "type": "Paragraph",
  "content": ["Some text content representing ideas expressed as words."]
}
```

```json
{
  "type": "Paragraph",
  "content": [
    "Some text with some",
    {
      "type": "Emphasis",
      "content": ["emphasized words"]
    },
    " and ",
    {
      "type": "Strong",
      "content": ["some strongly emphasized words"]
    }
  ]
}
```

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Paragraph.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Paragraph.schema.json)
- Python [`class Paragraph`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Paragraph)
- TypeScript [`interface Paragraph`](https://stencila.github.io/schema/ts/docs/interfaces/paragraph.html)
- R [`class Paragraph`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Paragraph`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Paragraph.html)

## Source

This documentation was generated from [Paragraph.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/Paragraph.schema.yaml).
