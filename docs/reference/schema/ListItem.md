# List Item

**A single item in a list.**

This is an implementation, and extension, of schema.org [`ListItem`](https://schema.org/ListItem). It extends schema.ord `ListItem` by adding `content` and `isChecked` properties. Analogues of `ListItem` in other schema include: - JATS XML `<list-item>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/list-item.html) - HTML [`<li>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li) - MDAST [`ListItem`](https://github.com/syntax-tree/mdast#listitem) - OpenDocument [`<text:list-item>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415154_253892949)

## Properties

| Name           | `@id`                                                           | Type                                                                                                 | Description                                                         | Inherited from          |
| -------------- | --------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- | ----------------------- |
| alternateNames | [schema:alternateName](https://schema.org/alternateName)        | Array of string                                                                                      | Alternate names (aliases) for the item.                             | [Thing](Thing.md)       |
| content        | [stencila:content](https://schema.stenci.la/content.jsonld)     | Array of [BlockContent](BlockContent.md) _or_ Array of [InlineContent](InlineContent.md)             | The content of the list item. See note [1](#notes).                 | [ListItem](ListItem.md) |
| description    | [schema:description](https://schema.org/description)            | Array of [BlockContent](BlockContent.md) _or_ Array of [InlineContent](InlineContent.md) _or_ string | A description of the item. See note [2](#notes).                    | [Thing](Thing.md)       |
| id             | [schema:id](https://schema.org/id)                              | string                                                                                               | The identifier for this item.                                       | [Entity](Entity.md)     |
| identifiers    | [schema:identifier](https://schema.org/identifier)              | Array of ([PropertyValue](PropertyValue.md) _or_ string)                                             | Any kind of identifier for any kind of Thing. See note [3](#notes). | [Thing](Thing.md)       |
| images         | [schema:image](https://schema.org/image)                        | Array of ([ImageObject](ImageObject.md) _or_ Format 'uri')                                           | Images of the item.                                                 | [Thing](Thing.md)       |
| isChecked      | [stencila:isChecked](https://schema.stenci.la/isChecked.jsonld) | boolean                                                                                              | A flag to indicate if this list item is checked.                    | [ListItem](ListItem.md) |
| item           | [schema:item](https://schema.org/item)                          | [Node](Node.md)                                                                                      | The item represented by this list item. See note [4](#notes).       | [ListItem](ListItem.md) |
| meta           | [stencila:meta](https://schema.stenci.la/meta.jsonld)           | object                                                                                               | Metadata associated with this item.                                 | [Entity](Entity.md)     |
| name           | [schema:name](https://schema.org/name)                          | string                                                                                               | The name of the item.                                               | [Thing](Thing.md)       |
| position       | [schema:position](https://schema.org/position)                  | integer                                                                                              | The position of the item in a series or sequence of items.          | [ListItem](ListItem.md) |
| url            | [schema:url](https://schema.org/url)                            | Format 'uri'                                                                                         | The URL of the item.                                                | [Thing](Thing.md)       |

## Notes

1. **content** : Use either `content` or `item`, not both.
2. **description** : Allows for the description to be an array of nodes (e.g. an array of inline content, or a couple of paragraphs), or a string. The `minItems` restriction avoids a string being coerced into an array with a single string item.
3. **identifiers** : Some identifiers have specific properties e.g the `issn` property for the `Periodical` type. These should be used in preference to this property which is intended for identifiers that do not yet have a specific property. Identifiers can be represented as strings, but using a `PropertyValue` will usually be better because it allows for `propertyID` (i.e. the type of identifier).
4. **item** : Use either `item` or `content`, not both.

## Examples

```json
{
  "type": "ListItem",
  "content": ["List Item Content"]
}
```

```json
{
  "type": "ListItem",
  "content": [
    "List Item Content",
    {
      "type": "List",
      "order": "ordered",
      "items": ["Nested Item One"]
    }
  ]
}
```

```json
{
  "type": "ListItem",
  "isChecked": true,
  "content": ["Completed todo item"]
}
```

## Related

- Parent: [Thing](Thing.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/ListItem.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/ListItem.schema.json)
- Python [`class ListItem`](https://stencila.github.io/schema/python/docs/types.html#schema.types.ListItem)
- TypeScript [`interface ListItem`](https://stencila.github.io/schema/ts/docs/interfaces/listitem.html)
- R [`class ListItem`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct ListItem`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.ListItem.html)

## Source

This documentation was generated from [ListItem.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/ListItem.schema.yaml).
