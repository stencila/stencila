---
title: SoftwareSourceCode
authors: []
---

include: ../built/SoftwareSourceCode.schema.md
:::
Computer programming source code. Example: Full (compile ready) solutions, code snippet samples, scripts, templates.

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

web page, scholarly article, etc. | array | | CreativeWork | content | The structured content of this creative work c.f. property \`text\`. | array | | CreativeWork | dateCreated | Date/time of creation. | | | CreativeWork | dateModified | Date/time of most recent modification. | | | CreativeWork | datePublished | Date of first publication. | | | CreativeWork | editors | Persons who edited the CreativeWork. | array | | CreativeWork | funders | Person or organisation that funded the CreativeWork. | array | | CreativeWork | isPartOf | An item or other CreativeWork that this CreativeWork is a part of. | | | CreativeWork | licenses | License documents that applies to this content, typically indicated by URL. | array | | CreativeWork | parts | Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more. | array | | CreativeWork | publisher | A publisher of the CreativeWork. | | | CreativeWork | text | The textual content of this creative work. | string | | CreativeWork | title | | string | | CreativeWork | version | | | | SoftwareSourceCode | codeRepository | Link to the repository where the un-compiled, human readable code and related code is located (SVN, github, CodePlex) | string | | SoftwareSourceCode | codeSampleType | What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template. | string | | SoftwareSourceCode | maintainers | The people or organizations who maintain the software. | array | | SoftwareSourceCode | programmingLanguage | The computer programming language. | string | | SoftwareSourceCode | runtimePlatform | Runtime platform or script interpreter dependencies (Example - Java v1, Python2.3, .Net Framework 3.0). | array | | SoftwareSourceCode | softwareRequirements | Component dependency requirements for application. This includes runtime environments and shared libraries that are not included in the application distribution package, but required to run the application (Examples include DirectX, Java or .NET runtime). | array | | SoftwareSourceCode | targetProducts | Target Operating System / Product to which the code applies. If applies to several versions, just the product name can be used. | array |
:::

Usually contains a link to a `codeRepository` where the code can be fetched from and then compiled. Normally using `SoftwareApplication` is preferred, if possible, however using `SoftwareSourceCode` allows for the installation of packages where no pre-built version is available.
