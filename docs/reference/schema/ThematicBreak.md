# Thematic Break

**A thematic break, such as a scene change in a story, a transition to another topic, or a new document.**

Analogues of `ThematicBreak` in other schema include: - JATS XML [`<hr>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/hr.html) - MDAST [`ThematicBreak`](https://github.com/syntax-tree/mdast#ThematicBreak) - OpenDocument OpenDocument [`<text:soft-page-break>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#element-text_soft-page-break)

## Properties

| Name | `@id`                                                 | Type   | Description                         | Inherited from      |
| ---- | ----------------------------------------------------- | ------ | ----------------------------------- | ------------------- |
| id   | [schema:id](https://schema.org/id)                    | string | The identifier for this item.       | [Entity](Entity.md) |
| meta | [stencila:meta](https://schema.stenci.la/meta.jsonld) | object | Metadata associated with this item. | [Entity](Entity.md) |

## Examples

```json
{
  "type": "ThematicBreak"
}
```

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/ThematicBreak.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/ThematicBreak.schema.json)
- Python [`class ThematicBreak`](https://stencila.github.io/schema/python/docs/types.html#schema.types.ThematicBreak)
- TypeScript [`interface ThematicBreak`](https://stencila.github.io/schema/ts/docs/interfaces/thematicbreak.html)
- R [`class ThematicBreak`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct ThematicBreak`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.ThematicBreak.html)

## Source

This documentation was generated from [ThematicBreak.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/ThematicBreak.schema.yaml).
