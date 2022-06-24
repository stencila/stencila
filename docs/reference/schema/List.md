# List

**A list of items.**

This is an implementation, and renaming, of schema.org [`ItemList`](https://schema.org/ItemList). Renaming was done as `List` was considered a more developer friendly alternative. Similarly, schema.org properties `itemListElement` and `itemListOrder` were renamed to `items` and `order`. Note that, as with every other such renaming in Stencila Schema, a mapping between names is defined and it is trivial to save Stencila Schema documents using the schema.org vocabulary if so desired. Analogues of `List` in other schema include: - JATS XML [`<list>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/list.html) - HTML [`<ul>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ul) and [`<ol>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ol) - MDAST [`List`](https://github.com/syntax-tree/mdast#list) - OpenDocument [`<text:list>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415148_253892949)

## Properties

| Name      | `@id`                                                        | Type                                   | Description                         | Inherited from      |
| --------- | ------------------------------------------------------------ | -------------------------------------- | ----------------------------------- | ------------------- |
| **items** | [schema:itemListElement](https://schema.org/itemListElement) | Array of [ListItem](ListItem.md)       | The items in the list               | [List](List.md)     |
| id        | [schema:id](https://schema.org/id)                           | string                                 | The identifier for this item.       | [Entity](Entity.md) |
| meta      | [stencila:meta](https://schema.stenci.la/meta.jsonld)        | object                                 | Metadata associated with this item. | [Entity](Entity.md) |
| order     | [schema:itemListOrder](https://schema.org/itemListOrder)     | 'Ascending', 'Descending', 'Unordered' | Type of ordering.                   | [List](List.md)     |

## Examples

```json
{
  "type": "List",
  "items": [
    {
      "type": "ListItem",
      "content": ["Item One"]
    },
    {
      "type": "ListItem",
      "content": ["Item Two"]
    },
    {
      "type": "ListItem",
      "content": ["Item Three"]
    }
  ]
}
```

```json
{
  "type": "List",
  "items": [
    {
      "type": "ListItem",
      "content": "Item One"
    },
    {
      "type": "ListItem",
      "content": [
        "This is a nested item",
        {
          "type": "List",
          "order": "ordered",
          "items": [
            {
              "type": "ListItem",
              "content": ["Nested Item One"]
            },
            {
              "type": "ListItem",
              "content": ["Nested Item Two"]
            },
            {
              "type": "ListItem",
              "content": ["Nested Item Three"]
            }
          ]
        }
      ]
    },
    {
      "type": "ListItem",
      "content": ["Item Three"]
    }
  ]
}
```

```json
{
  "type": "List",
  "items": [
    {
      "type": "ListItem",
      "checked": false,
      "content": ["Todo item"]
    },
    {
      "type": "ListItem",
      "checked": true,
      "content": ["Completed todo item"]
    }
  ]
}
```

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/ItemList.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/List.schema.json)
- Python [`class List`](https://stencila.github.io/schema/python/docs/types.html#schema.types.ItemList)
- TypeScript [`interface List`](https://stencila.github.io/schema/ts/docs/interfaces/itemlist.html)
- R [`class List`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct List`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.ItemList.html)

## Source

This documentation was generated from [List.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/List.schema.yaml).
