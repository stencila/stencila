---
title: Cite
authors: []
---

include: ../built/Cite.schema.md
:::
A reference to a CreativeWork that is cited in another CreativeWork.

## Properties

| **type _(required)_** | `enum<`​`Cite`​`>`                                                              | The name of the type and all descendant types.                                                                          | [Entity](./Entity.html)             |
| --------------------- | ------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| alternateNames        | `array<`​`string`​`>`                                                           | Alternate names (aliases) for the item.                                                                                 | [Thing](./Thing.html)               |
| authors               | `array<`​[`Person`](./Person.html) \| [`Organization`](./Organization.html)​`>` | The authors of this creative work.                                                                                      | [CreativeWork](./CreativeWork.html) |
| citationMode          | `enum<`​`normal` \| `suppressAuthor`​`>`                                        | How the cite is rendered in the surrounding text.                                                                       | [Cite](./Cite.html)                 |
| citations             | `array<`​`string` \| [`CreativeWorkTypes`](./CreativeWorkTypes.html)​`>`        | Citations or references to other creative works, such as another publication, web page, scholarly article, etc.         | [CreativeWork](./CreativeWork.html) |
| content               | `array<`​[`Node`](./Node.html)​`>`                                              | The structured content of this creative work c.f. property \`text\`.                                                    | [CreativeWork](./CreativeWork.html) |
| dateCreated           | `string<date>` \| `string<date-time>`                                           | Date/time of creation.                                                                                                  | [CreativeWork](./CreativeWork.html) |
| dateModified          | `string<date>` \| `string<date-time>`                                           | Date/time of most recent modification.                                                                                  | [CreativeWork](./CreativeWork.html) |
| datePublished         | `string<date>` \| `string<date-time>`                                           | Date of first publication.                                                                                              | [CreativeWork](./CreativeWork.html) |
| description           | `string`                                                                        | A description of the item.                                                                                              | [Thing](./Thing.html)               |
| editors               | `array<`​[`Person`](./Person.html)​`>`                                          | Persons who edited the CreativeWork.                                                                                    | [CreativeWork](./CreativeWork.html) |
| funders               | `array<`​[`Person`](./Person.html) \| [`Organization`](./Organization.html)​`>` | Person or organisation that funded the CreativeWork.                                                                    | [CreativeWork](./CreativeWork.html) |
| id                    | `string`                                                                        | The identifier for this item.                                                                                           | [Entity](./Entity.html)             |
| isPartOf              | [`CreativeWorkTypes`](./CreativeWorkTypes.html)                                 | An item or other CreativeWork that this CreativeWork is a part of.                                                      | [CreativeWork](./CreativeWork.html) |
| licenses              | `array<`​`string<uri>` \| [`CreativeWorkTypes`](./CreativeWorkTypes.html)​`>`   | License documents that applies to this content, typically indicated by URL.                                             | [CreativeWork](./CreativeWork.html) |
| meta                  | `object`                                                                        | Metadata associated with this item.                                                                                     | [Entity](./Entity.html)             |
| name                  | `string`                                                                        | The name of the item.                                                                                                   | [Thing](./Thing.html)               |
| pageEnd               | `string` \| `integer`                                                           | The page on which the work ends; for example "138" or "xvi".                                                            | [Cite](./Cite.html)                 |
| pageStart             | `string` \| `integer`                                                           | The page on which the work starts; for example "135" or "xiii".                                                         | [Cite](./Cite.html)                 |
| pagination            | `string`                                                                        | Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".                   | [Cite](./Cite.html)                 |
| parts                 | `array<`​[`CreativeWorkTypes`](./CreativeWorkTypes.html)​`>`                    | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | [CreativeWork](./CreativeWork.html) |
| prefix                | `string`                                                                        | A prefix to show before the citation.                                                                                   | [Cite](./Cite.html)                 |
| publisher             | [`Person`](./Person.html) \| [`Organization`](./Organization.html)              | A publisher of the CreativeWork.                                                                                        | [CreativeWork](./CreativeWork.html) |
| suffix                | `string`                                                                        | A suffix to show after the citation.                                                                                    | [Cite](./Cite.html)                 |
| target                | `string`                                                                        | The target of the citation (URL or reference ID).                                                                       | [Cite](./Cite.html)                 |
| text                  | `string`                                                                        | The textual content of this creative work.                                                                              | [CreativeWork](./CreativeWork.html) |
| title                 | `string`                                                                        | The title of the creative work.                                                                                         | [CreativeWork](./CreativeWork.html) |
| url                   | `string<uri>`                                                                   | The URL of the item.                                                                                                    | [Thing](./Thing.html)               |
| version               | `string` \| `number`                                                            | The version of the creative work.                                                                                       | [CreativeWork](./CreativeWork.html) |

:::

A `Cite` node is used within a [`CreativeWork`](./CreativeWork.html), usually an [`Article`](./Article.html), to refer to an other `CreativeWork`. Often a `Cite` will be associated with other citations, in a [`CiteGroup`](../CiteGroup.html).

# Examples

The following example of a (very) short article, shows how a `Cite` is used to refer to another `Article` in the article's `references`.

```json import=ex1
{
  "type": "Article",
  "title": "An example of using the Cite node type",
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
        "A citation of ",
        {
          "type": "Cite",
          "target": "some-one-else-1991"
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
    }
  ]
}
```

# Encodings

```md
---
title: An example of using the Cite node type
authors:
  - type: Person
    #...
references:
  - type: Article
    id: some-one-else-1991
    #...
---

A citation of @some-one-else-1991.
```
