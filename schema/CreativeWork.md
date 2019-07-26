---
title: Creative Work
authors: []
---

include: ../built/CreativeWork.schema.md
:::
The most generic kind of creative work, including books, movies, photographs, software programs, etc. https&#x3A;//schema.org/CreativeWork

| Entity       | type           | The name of the type and all descendant types.                                | string |
| ------------ | -------------- | ----------------------------------------------------------------------------- | ------ |
| Entity       | id             | The identifier for this item.                                                 | string |
| Thing        | alternateNames | Alternate names (aliases) for the item.                                       | array  |
| Thing        | description    | A description of the item.                                                    | string |
| Thing        | meta           | Metadata associated with this item.                                           | object |
| Thing        | name           | The name of the item.                                                         | string |
| Thing        | url            | The URL of the item.                                                          | string |
| CreativeWork | authors        | The authors of this this creative work.                                       | array  |
| CreativeWork | citations      | Citations or references to other creative works, such as another publication, |        |

web page, scholarly article, etc. | array | | CreativeWork | content | The structured content of this creative work c.f. property \`text\`. | array | | CreativeWork | dateCreated | Date/time of creation. | | | CreativeWork | dateModified | Date/time of most recent modification. | | | CreativeWork | datePublished | Date of first publication. | | | CreativeWork | editors | Persons who edited the CreativeWork. | array | | CreativeWork | funders | Person or organisation that funded the CreativeWork. | array | | CreativeWork | isPartOf | An item or other CreativeWork that this CreativeWork is a part of. | | | CreativeWork | licenses | License documents that applies to this content, typically indicated by URL. | array | | CreativeWork | parts | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | array | | CreativeWork | publisher | A publisher of the CreativeWork. | | | CreativeWork | text | The textual content of this creative work. | string | | CreativeWork | title | | string | | CreativeWork | version | | |
:::

The most generic kind of creative work, including books, movies, photographs, software programs, etc.

# Examples

For example usage see:

- [`Article`](/schema/Article)
- [`Collection`](/schema/Collection)
- [`Datatable`](/schema/Datatable)
- [`Media Object`](/schema/MediaObject)
- [`Software Application`](/schema/SoftwareApplication)
- [`Table`](/schema/Table)

```json
{
"type": "CreativeWork",
"authors":[
    {"type": "Person",   "givenNames": ["Marie"], "familyNames": ["Skłodowska", "Curie"]}
    ],
"title": "Radioactivity",
"citations": ["Marie Curie's century-old radioactive notebook still requires lead box"],
"content":[
     {"type": "Node"} ],
"datePublished": "2019-05-20",
"editors":[
    {"type": "Person",   "givenNames": ["John"], "familyNames": ["Smith"]}
    ],
"funders": [
    {"type": "Person",   "givenNames": ["Joanna"], "familyNames": ["Smith"]}
    ],
"isPartOf": {
    "type": "CreativeWork",
    "title": "The Great Works of Marie Skłodowska-Curie"
},
"licenses":[
    {"https://opensource.org/licenses/MIT"}
],
"publisher": "Random House",
"text": "Sample text",
"version": "4"
}
```
