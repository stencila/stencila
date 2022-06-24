# Cite Group

**A group of Cite nodes.**

This type allows you to group associated citations together. When some content in a [`Creative Work`](./CreativeWork) cites more than one reference for a particular piece of text, use a `CiteGroup` to encapsulate multiple [`Cite`](./Cite) nodes. At present we do not give a `citationMode` property to a `CiteGroup` since they will almost always be parenthetical as opposed to narrative. In other words, it usually only makes sense for individual `Cite` nodes to be narrative (although they may be connected together within `content` using words such as "and").

This schema type is marked as **unstable** ⚠️ and is subject to change.

## Properties

| Name      | `@id`                                                        | Type                     | Description                                                        | Inherited from            |
| --------- | ------------------------------------------------------------ | ------------------------ | ------------------------------------------------------------------ | ------------------------- |
| **items** | [schema:itemListElement](https://schema.org/itemListElement) | Array of [Cite](Cite.md) | One or more `Cite`s to be referenced in the same surrounding text. | [CiteGroup](CiteGroup.md) |
| id        | [schema:id](https://schema.org/id)                           | string                   | The identifier for this item.                                      | [Entity](Entity.md)       |
| meta      | [stencila:meta](https://schema.stenci.la/meta.jsonld)        | object                   | Metadata associated with this item.                                | [Entity](Entity.md)       |

## Examples

```json
{
  "type": "Article",
  "title": "An example of using the CiteGroup node type",
  "authors": [
    {
      "type": "Person",
      "givenNames": ["Joe"],
      "familyNames": ["Bloggs"]
    }
  ],
  "content": [
    {
      "type": "Paragraph",
      "content": [
        "Citing two articles ",
        {
          "type": "CiteGroup",
          "items": [
            {
              "type": "Cite",
              "target": "some-one-else-1991"
            },
            {
              "type": "Cite",
              "target": "updated-works-2009"
            }
          ]
        },
        "."
      ]
    }
  ],
  "references": [
    {
      "type": "Article",
      "id": "some-one-else-1991",
      "title": "Another article by someone else",
      "authors": [
        {
          "type": "Person",
          "givenNames": ["Some", "One"],
          "familyNames": ["Else"]
        }
      ],
      "datePublished": "1991"
    },
    {
      "type": "Article",
      "id": "update-works-2009",
      "title": "A Better Updated Work",
      "authors": [
        {
          "type": "Person",
          "givenNames": ["Some", "Better"],
          "familyNames": ["Person"]
        }
      ],
      "datePublished": "2009"
    }
  ]
}
```

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/CiteGroup.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/CiteGroup.schema.json)
- Python [`class CiteGroup`](https://stencila.github.io/schema/python/docs/types.html#schema.types.CiteGroup)
- TypeScript [`interface CiteGroup`](https://stencila.github.io/schema/ts/docs/interfaces/citegroup.html)
- R [`class CiteGroup`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct CiteGroup`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.CiteGroup.html)

## Source

This documentation was generated from [CiteGroup.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/CiteGroup.schema.yaml).
