# Heading

**A heading.**

Analogues of `Delete` in other schema include: - HTML [`<h1>` to `<h6>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h1) - JATS XML [`<title>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/title.html) - Pandoc [`Header`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L233)

## Properties

| Name        | `@id`                                                       | Type                                       | Description                         | Inherited from        |
| ----------- | ----------------------------------------------------------- | ------------------------------------------ | ----------------------------------- | --------------------- |
| **content** | [stencila:content](https://schema.stenci.la/content.jsonld) | Array of [InlineContent](InlineContent.md) | Content of the heading.             | [Heading](Heading.md) |
| depth       | [stencila:depth](https://schema.stenci.la/depth.jsonld)     | integer                                    | The depth of the heading.           | [Heading](Heading.md) |
| id          | [schema:id](https://schema.org/id)                          | string                                     | The identifier for this item.       | [Entity](Entity.md)   |
| meta        | [stencila:meta](https://schema.stenci.la/meta.jsonld)       | object                                     | Metadata associated with this item. | [Entity](Entity.md)   |

## Examples

```json
{
  "type": "Heading",
  "depth": 2,
  "content": ["Secondary Heading"]
}
```

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Heading.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Heading.schema.json)
- Python [`class Heading`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Heading)
- TypeScript [`interface Heading`](https://stencila.github.io/schema/ts/docs/interfaces/heading.html)
- R [`class Heading`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Heading`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Heading.html)

## Source

This documentation was generated from [Heading.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/Heading.schema.yaml).
