---
title: Cite Group
authors: []
---

include: ../public/CiteGroup.schema.md
:::
A group of `Cite` nodes

## Properties

| **items _(required)_** | `array<`​[`Cite`](./Cite.html)​`>` | One or more Cites to be referenced in the same surrounding text. | [CiteGroup](./CiteGroup.html) |
| ---------------------- | ---------------------------------- | ---------------------------------------------------------------- | ----------------------------- |
| **type _(required)_**  | `enum<`​`CiteGroup`​`>`            | The name of the type and all descendant types.                   | [Entity](./Entity.html)       |
| id                     | `string`                           | The identifier for this item.                                    | [Entity](./Entity.html)       |
| meta                   | `object`                           | Metadata associated with this item.                              | [Entity](./Entity.html)       |

:::

When some content in a [`Creative Work`](./CreativeWork.html) cites more than one reference for a particular piece of
text, use a `CiteGroup` to encapsulate multiple [`Cite`](./Cite.html) nodes.

# Examples

The following example is a very short article that groups multiple citations together.

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
            { "type": "Cite", "target": "some-one-else-1991" },
            { "type": "Cite", "target": "updated-works-2009" }
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
