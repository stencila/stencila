---
title: Periodical
authors: []
---

# Periodical

include: ../built/Periodical.schema.md
:::
A periodical publication.

## Properties

| **type _(required)_** | `enum<`​`Periodical`​`>`                                                        | The name of the type and all descendant types.                                                                          | [Entity](./Entity.html)             |
| --------------------- | ------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| alternateNames        | `array<`​`string`​`>`                                                           | Alternate names (aliases) for the item.                                                                                 | [Thing](./Thing.html)               |
| authors               | `array<`​[`Person`](./Person.html) \| [`Organization`](./Organization.html)​`>` | The authors of this creative work.                                                                                      | [CreativeWork](./CreativeWork.html) |
| citations             | `array<`​`string` \| [`CreativeWork`](./CreativeWork.html)​`>`                  | Citations or references to other creative works, such as another publication, web page, scholarly article, etc.         | [CreativeWork](./CreativeWork.html) |
| content               | `array<`​[`Node`](./Node.html)​`>`                                              | The structured content of this creative work c.f. property \`text\`.                                                    | [CreativeWork](./CreativeWork.html) |
| dateCreated           | `string<date>` \| `string<date-time>`                                           | Date/time of creation.                                                                                                  | [CreativeWork](./CreativeWork.html) |
| dateEnd               | `string<date>` \| `string<date-time>`                                           | The date this Periodical ceased publication.                                                                            | [Periodical](./Periodical.html)     |
| dateModified          | `string<date>` \| `string<date-time>`                                           | Date/time of most recent modification.                                                                                  | [CreativeWork](./CreativeWork.html) |
| datePublished         | `string<date>` \| `string<date-time>`                                           | Date of first publication.                                                                                              | [CreativeWork](./CreativeWork.html) |
| dateStart             | `string<date>` \| `string<date-time>`                                           | The date this Periodical was first published.                                                                           | [Periodical](./Periodical.html)     |
| description           | `string`                                                                        | A description of the item.                                                                                              | [Thing](./Thing.html)               |
| editors               | `array<`​[`Person`](./Person.html)​`>`                                          | Persons who edited the CreativeWork.                                                                                    | [CreativeWork](./CreativeWork.html) |
| funders               | `array<`​[`Person`](./Person.html) \| [`Organization`](./Organization.html)​`>` | Person or organisation that funded the CreativeWork.                                                                    | [CreativeWork](./CreativeWork.html) |
| id                    | `string`                                                                        | The identifier for this item.                                                                                           | [Entity](./Entity.html)             |
| isPartOf              | [`CreativeWork`](./CreativeWork.html)                                           | An item or other CreativeWork that this CreativeWork is a part of.                                                      | [CreativeWork](./CreativeWork.html) |
| issn                  | `array<`​`string`​`>`                                                           | The International Standard Serial Number (ISSN) that identifies this serial publication.                                | [Periodical](./Periodical.html)     |
| licenses              | `array<`​`string<uri>` \| [`CreativeWork`](./CreativeWork.html)​`>`             | License documents that applies to this content, typically indicated by URL.                                             | [CreativeWork](./CreativeWork.html) |
| meta                  | `object`                                                                        | Metadata associated with this item.                                                                                     | [Entity](./Entity.html)             |
| name                  | `string`                                                                        | The name of the item.                                                                                                   | [Thing](./Thing.html)               |
| parts                 | `array<`​[`CreativeWork`](./CreativeWork.html)​`>`                              | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | [CreativeWork](./CreativeWork.html) |
| publisher             | [`Person`](./Person.html) \| [`Organization`](./Organization.html)              | A publisher of the CreativeWork.                                                                                        | [CreativeWork](./CreativeWork.html) |
| text                  | `string`                                                                        | The textual content of this creative work.                                                                              | [CreativeWork](./CreativeWork.html) |
| title                 | `string`                                                                        | The title of the creative work.                                                                                         | [CreativeWork](./CreativeWork.html) |
| url                   | `string<uri>`                                                                   | The URL of the item.                                                                                                    | [Thing](./Thing.html)               |
| version               | `string` \| `number`                                                            | The version of the creative work.                                                                                       | [CreativeWork](./CreativeWork.html) |

:::

A publication in any medium issued in successive parts bearing numerical or chronological designations and intended,
such as a magazine, scholarly journal, or newspaper to continue indefinitely.

## Examples

An example of the journal Nature is below. The `dateStart` is the date it was first published. The first `issn` numbers
is for the printed journal and the second for the online edition.

```json validate
{
  "type": "Periodical",
  "title": "Nature",
  "issn": ["0028-0836", "1476-4687"],
  "dateStart": "1869-11-04",
  "url": "https://www.nature.com/"
}
```
