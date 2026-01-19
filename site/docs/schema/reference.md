---
title: Reference
description: A reference to a creative work, including books, movies, photographs, software programs, etc.
---

# Properties

The `Reference` type has these properties:

| Name           | Description                                                                                           | Type                                                                 | Inherited from          | `JSON-LD @id`                                            | Aliases                          |
| -------------- | ----------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- | -------------------------------------------------------- | -------------------------------- |
| `id`           | The identifier for this item.                                                                         | [`String`](./string.md)                                              | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)                     | -                                |
| `workType`     | The type of `CreativeWork` being referenced (e.g. Article, Book, Dataset).                            | [`CreativeWorkType`](./creative-work-type.md)                        | -                       | `stencila:workType`                                      | `work-type`, `work_type`         |
| `doi`          | The Digital Object Identifier (https://doi.org/) of the work being referenced.                        | [`String`](./string.md)                                              | -                       | `stencila:doi`                                           | -                                |
| `authors`      | The authors of the work.                                                                              | [`Author`](./author.md)*                                             | -                       | [`schema:author`](https://schema.org/author)             | `author`                         |
| `editors`      | People who edited the referenced work.                                                                | [`Person`](./person.md)*                                             | -                       | [`schema:editor`](https://schema.org/editor)             | `editor`                         |
| `publisher`    | A publisher of the referenced work.                                                                   | [`Person`](./person.md) \| [`Organization`](./organization.md)       | -                       | [`schema:publisher`](https://schema.org/publisher)       | -                                |
| `date`         | Date of first publication.                                                                            | [`Date`](./date.md)                                                  | -                       | [`schema:date`](https://schema.org/date)                 | -                                |
| `title`        | The title of the referenced work.                                                                     | [`Inline`](./inline.md)*                                             | -                       | [`schema:headline`](https://schema.org/headline)         | `headline`                       |
| `isPartOf`     | Another `Reference` that this reference is a part of.                                                 | [`Reference`](./reference.md)                                        | -                       | [`schema:isPartOf`](https://schema.org/isPartOf)         | `is-part-of`, `is_part_of`       |
| `volumeNumber` | Identifies the volume of publication or multi-part work; for example, "iii" or "2".                   | [`Integer`](./integer.md) \| [`String`](./string.md)                 | -                       | [`schema:volumeNumber`](https://schema.org/volumeNumber) | `volume-number`, `volume_number` |
| `issueNumber`  | Identifies the issue of a serial publication; for example, "3" or "12".                               | [`Integer`](./integer.md) \| [`String`](./string.md)                 | -                       | [`schema:issueNumber`](https://schema.org/issueNumber)   | `issue-number`, `issue_number`   |
| `pageStart`    | The page on which the article starts; for example "135" or "xiii".                                    | [`Integer`](./integer.md) \| [`String`](./string.md)                 | -                       | [`schema:pageStart`](https://schema.org/pageStart)       | `page-start`, `page_start`       |
| `pageEnd`      | The page on which the article ends; for example "138" or "xvi".                                       | [`Integer`](./integer.md) \| [`String`](./string.md)                 | -                       | [`schema:pageEnd`](https://schema.org/pageEnd)           | `page-end`, `page_end`           |
| `pagination`   | Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55". | [`String`](./string.md)                                              | -                       | [`schema:pagination`](https://schema.org/pagination)     | -                                |
| `version`      | The version/edition of the referenced work.                                                           | [`String`](./string.md) \| [`Number`](./number.md)                   | -                       | [`schema:version`](https://schema.org/version)           | -                                |
| `identifiers`  | Any kind of identifier for the referenced work.                                                       | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | -                       | [`schema:identifier`](https://schema.org/identifier)     | `identifier`                     |
| `url`          | The URL of the referenced work.                                                                       | [`String`](./string.md)                                              | -                       | [`schema:url`](https://schema.org/url)                   | -                                |
| `text`         | Plain text representation of the referenced work.                                                     | [`String`](./string.md)                                              | -                       | [`schema:text`](https://schema.org/text)                 | -                                |
| `content`      | A rendering of the reference using the citation style of the document.                                | [`Inline`](./inline.md)*                                             | -                       | `stencila:content`                                       | -                                |

# Related

The `Reference` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `Reference` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                            | Notes |
| ------------------------------------------------ | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                    |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |                                    |
| [JATS](../formats/jats.md)                       |              |              |                                    |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              | Encoded using implemented function |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |                                    |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |                                    |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |                                    |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |                                    |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                    |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                    |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                    |
| [Citation File Format](../formats/cff.md)        |              |              |                                    |
| [CSV](../formats/csv.md)                         |              |              |                                    |
| [TSV](../formats/tsv.md)                         |              |              |                                    |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                    |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                    |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                    |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                    |
| [Directory](../formats/directory.md)             |              |              |                                    |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                    |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                    |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                    |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                    |
| [Email HTML](../formats/email.html.md)           |              |              |                                    |
| [MJML](../formats/mjml.md)                       |              |              |                                    |

# Bindings

The `Reference` type is represented in:

- [JSON-LD](https://stencila.org/Reference.jsonld)
- [JSON Schema](https://stencila.org/Reference.schema.json)
- Python class [`Reference`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/reference.py)
- Rust struct [`Reference`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/reference.rs)
- TypeScript class [`Reference`](https://github.com/stencila/stencila/blob/main/ts/src/types/Reference.ts)

# Source

This documentation was generated from [`Reference.yaml`](https://github.com/stencila/stencila/blob/main/schema/Reference.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
