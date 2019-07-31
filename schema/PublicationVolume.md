---
title: Publication Volume
authors: []
---

# Publication Volume

include: ../built/PublicationVolume.schema.md
:::
A part of a successively published publication such as a periodical or multi-volume work.

## Properties

| **type _(required)_** | `enum<`​`PublicationVolume`​`>`                                                 | The name of the type and all descendant types.                                                                          | [Entity](./Entity.html)                       |
| --------------------- | ------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- |
| alternateNames        | `array<`​`string`​`>`                                                           | Alternate names (aliases) for the item.                                                                                 | [Thing](./Thing.html)                         |
| authors               | `array<`​[`Person`](./Person.html) \| [`Organization`](./Organization.html)​`>` | The authors of this creative work.                                                                                      | [CreativeWork](./CreativeWork.html)           |
| citations             | `array<`​`string` \| [`CreativeWork`](./CreativeWork.html)​`>`                  | Citations or references to other creative works, such as another publication, web page, scholarly article, etc.         | [CreativeWork](./CreativeWork.html)           |
| content               | `array<`​[`Node`](./Node.html)​`>`                                              | The structured content of this creative work c.f. property \`text\`.                                                    | [CreativeWork](./CreativeWork.html)           |
| dateCreated           | `string<date>` \| `string<date-time>`                                           | Date/time of creation.                                                                                                  | [CreativeWork](./CreativeWork.html)           |
| dateModified          | `string<date>` \| `string<date-time>`                                           | Date/time of most recent modification.                                                                                  | [CreativeWork](./CreativeWork.html)           |
| datePublished         | `string<date>` \| `string<date-time>`                                           | Date of first publication.                                                                                              | [CreativeWork](./CreativeWork.html)           |
| description           | `string`                                                                        | A description of the item.                                                                                              | [Thing](./Thing.html)                         |
| editors               | `array<`​[`Person`](./Person.html)​`>`                                          | Persons who edited the CreativeWork.                                                                                    | [CreativeWork](./CreativeWork.html)           |
| funders               | `array<`​[`Person`](./Person.html) \| [`Organization`](./Organization.html)​`>` | Person or organisation that funded the CreativeWork.                                                                    | [CreativeWork](./CreativeWork.html)           |
| id                    | `string`                                                                        | The identifier for this item.                                                                                           | [Entity](./Entity.html)                       |
| isPartOf              | [`CreativeWork`](./CreativeWork.html)                                           | An item or other CreativeWork that this CreativeWork is a part of.                                                      | [CreativeWork](./CreativeWork.html)           |
| licenses              | `array<`​`string<uri>` \| [`CreativeWork`](./CreativeWork.html)​`>`             | License documents that applies to this content, typically indicated by URL.                                             | [CreativeWork](./CreativeWork.html)           |
| meta                  | `object`                                                                        | Metadata associated with this item.                                                                                     | [Entity](./Entity.html)                       |
| name                  | `string`                                                                        | The name of the item.                                                                                                   | [Thing](./Thing.html)                         |
| pageEnd               | `string` \| `integer`                                                           | The page on which the work ends; for example "138" or "xvi".                                                            | [PublicationVolume](./PublicationVolume.html) |
| pageStart             | `string` \| `integer`                                                           | The page on which the work starts; for example "135" or "xiii".                                                         | [PublicationVolume](./PublicationVolume.html) |
| pagination            | `string`                                                                        | Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55" or "10-12, 46-49". | [PublicationVolume](./PublicationVolume.html) |
| parts                 | `array<`​[`CreativeWork`](./CreativeWork.html)​`>`                              | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | [CreativeWork](./CreativeWork.html)           |
| publisher             | [`Person`](./Person.html) \| [`Organization`](./Organization.html)              | A publisher of the CreativeWork.                                                                                        | [CreativeWork](./CreativeWork.html)           |
| text                  | `string`                                                                        | The textual content of this creative work.                                                                              | [CreativeWork](./CreativeWork.html)           |
| title                 | `string`                                                                        | The title of the creative work.                                                                                         | [CreativeWork](./CreativeWork.html)           |
| url                   | `string<uri>`                                                                   | The URL of the item.                                                                                                    | [Thing](./Thing.html)                         |
| version               | `string` \| `number`                                                            | The version of the creative work.                                                                                       | [CreativeWork](./CreativeWork.html)           |
| volumeNumber          | `string` \| `integer`                                                           | Identifies the volume of publication or multi-part work; for example, "iii" or "2".                                     | [PublicationVolume](./PublicationVolume.html) |

:::

A part of a successively published publication such as a periodical or multi-volume work, often numbered. It may
represent a time span, such as a year.

# Examples

A volume of Nature, in Stencila JSON.

```json validate
{
  "type": "PublicationVolume",
  "volumeNumber": 571,
  "isPartOf": {
    "type": "Periodical",
    "title": "Nature"
  }
}
```
