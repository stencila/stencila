---
title: SoftwareApplication
authors: []
---

include: ../public/SoftwareApplication.schema.md
:::
A software application.

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

web page, scholarly article, etc. | array | | CreativeWork | content | The structured content of this creative work c.f. property \`text\`. | array | | CreativeWork | dateCreated | Date/time of creation. | | | CreativeWork | dateModified | Date/time of most recent modification. | | | CreativeWork | datePublished | Date of first publication. | | | CreativeWork | editors | Persons who edited the CreativeWork. | array | | CreativeWork | funders | Person or organisation that funded the CreativeWork. | array | | CreativeWork | isPartOf | An item or other CreativeWork that this CreativeWork is a part of. | | | CreativeWork | licenses | License documents that applies to this content, typically indicated by URL. | array | | CreativeWork | parts | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | array | | CreativeWork | publisher | A publisher of the CreativeWork. | | | CreativeWork | text | The textual content of this creative work. | string | | CreativeWork | title | | string | | CreativeWork | version | | | | SoftwareApplication | softwareRequirements | Component dependency requirements for application. This includes runtime environments and shared libraries that are not included in the application distribution package, but required to run the application (Examples: DirectX, Java or .NET runtime). | array | | SoftwareApplication | softwareVersion | Version of the software instance. | string |
:::

`SoftwareApplication` represents a software application that can be located by name and version. It may be fetched in binary format or some package manager (npm, pip, etc) could fetch and compile the source code.

## Parsing/Interpretation

Although by using the `softwareRequirements` property it is possible to represent a full hierarchy of required packages, normally it is adequate to only list the immediate dependencies and allow the system/language package manager to resolve the rest.
