# Link

**A hyperlink to other pages, sections within the same document, resources, or any URL.**

Analogues of `Link` in other schema include: - HTML [`<a>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a) - JATS XML [`<ext-link>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/ext-link.html) - MDAST [`Link`](https://github.com/syntax-tree/mdast#link) - OpenDocument [`<text:a>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415212_253892949) - Pandoc [`Link`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L270)

This schema type is marked as **unstable** ⚠️ and is subject to change.

## Properties

| Name        | `@id`                                                             | Type                                       | Description                                                                           | Inherited from      |
| ----------- | ----------------------------------------------------------------- | ------------------------------------------ | ------------------------------------------------------------------------------------- | ------------------- |
| **content** | [stencila:content](https://schema.stenci.la/content.jsonld)       | Array of [InlineContent](InlineContent.md) | The textual content of the link.                                                      | [Link](Link.md)     |
| **target**  | [stencila:target](https://schema.stenci.la/target.jsonld)         | Format 'uri-reference'                     | The target of the link.                                                               | [Link](Link.md)     |
| exportFrom  | [stencila:exportFrom](https://schema.stenci.la/exportFrom.jsonld) | string                                     | A compilation directive giving the name of the variable to export to the link target. | [Link](Link.md)     |
| id          | [schema:id](https://schema.org/id)                                | string                                     | The identifier for this item.                                                         | [Entity](Entity.md) |
| importTo    | [stencila:importTo](https://schema.stenci.la/importTo.jsonld)     | string                                     | A compilation directive giving the name of the variable to import the link target as. | [Link](Link.md)     |
| meta        | [stencila:meta](https://schema.stenci.la/meta.jsonld)             | object                                     | Metadata associated with this item.                                                   | [Entity](Entity.md) |
| relation    | [schema:linkRelationship](https://schema.org/linkRelationship)    | string                                     | The relation between the target and the current thing.                                | [Link](Link.md)     |
| title       | [schema:headline](https://schema.org/headline)                    | string                                     | A title for the link. See note [1](#notes).                                           | [Link](Link.md)     |

## Notes

1. **title** : This property is analogous to the HTML `title` global attribute which represents ["advisory information related to the element"](https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/title)

## Examples

```json
{
  "type": "Link",
  "content": ["Stencila’s website"],
  "target": "https://stenci.la"
}
```

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Link.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Link.schema.json)
- Python [`class Link`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Link)
- TypeScript [`interface Link`](https://stencila.github.io/schema/ts/docs/interfaces/link.html)
- R [`class Link`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Link`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Link.html)

## Source

This documentation was generated from [Link.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/Link.schema.yaml).
