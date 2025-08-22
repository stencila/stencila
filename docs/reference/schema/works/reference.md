---
title: Reference
description: A reference to a creative work, including books, movies, photographs, software programs, etc.
config:
  publish:
    ghost:
      type: post
      slug: reference
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Works
---

# Properties

The `Reference` type has these properties:

| Name           | Description                                                                                           | Type                                                                                                                                                       | Inherited from                                                     | `JSON-LD @id`                                            | Aliases                          |
| -------------- | ----------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------------------- | -------------------------------- |
| `id`           | The identifier for this item.                                                                         | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                     | -                                |
| `workType`     | The type of `CreativeWork` being referenced(e.g. Article, Book, Dataset).                             | [`CreativeWorkType`](https://stencila.ghost.io/docs/reference/schema/creative-work-type)                                                                   | -                                                                  | `stencila:workType`                                      | `work-type`, `work_type`         |
| `doi`          | The Digital Object Identifier (https://doi.org/) of the work being referenced.                        | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                  | `stencila:doi`                                           | -                                |
| `authors`      | The authors of the work.                                                                              | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                                                                                        | -                                                                  | [`schema:author`](https://schema.org/author)             | `author`                         |
| `editors`      | People who edited the referenced work.                                                                | [`Person`](https://stencila.ghost.io/docs/reference/schema/person)*                                                                                        | -                                                                  | [`schema:editor`](https://schema.org/editor)             | `editor`                         |
| `publisher`    | A publisher of the referenced work.                                                                   | [`Person`](https://stencila.ghost.io/docs/reference/schema/person) \| [`Organization`](https://stencila.ghost.io/docs/reference/schema/organization)       | -                                                                  | [`schema:publisher`](https://schema.org/publisher)       | -                                |
| `date`         | Date of first publication.                                                                            | [`Date`](https://stencila.ghost.io/docs/reference/schema/date)                                                                                             | -                                                                  | [`schema:date`](https://schema.org/date)                 | -                                |
| `title`        | The title of the referenced work.                                                                     | [`Inline`](https://stencila.ghost.io/docs/reference/schema/inline)*                                                                                        | -                                                                  | [`schema:headline`](https://schema.org/headline)         | `headline`                       |
| `isPartOf`     | Another `Reference` that this reference is a part of.                                                 | [`Reference`](https://stencila.ghost.io/docs/reference/schema/reference)                                                                                   | -                                                                  | [`schema:isPartOf`](https://schema.org/isPartOf)         | `is-part-of`, `is_part_of`       |
| `volumeNumber` | Identifies the volume of publication or multi-part work; for example, "iii" or "2".                   | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string)                 | -                                                                  | [`schema:volumeNumber`](https://schema.org/volumeNumber) | `volume-number`, `volume_number` |
| `issueNumber`  | Identifies the issue of a serial publication; for example, "3" or "12".                               | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string)                 | -                                                                  | [`schema:issueNumber`](https://schema.org/issueNumber)   | `issue-number`, `issue_number`   |
| `pageStart`    | The page on which the article starts; for example "135" or "xiii".                                    | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string)                 | -                                                                  | [`schema:pageStart`](https://schema.org/pageStart)       | `page-start`, `page_start`       |
| `pageEnd`      | The page on which the article ends; for example "138" or "xvi".                                       | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string)                 | -                                                                  | [`schema:pageEnd`](https://schema.org/pageEnd)           | `page-end`, `page_end`           |
| `pagination`   | Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55". | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                  | [`schema:pagination`](https://schema.org/pagination)     | -                                |
| `version`      | The version/edition of the referenced work.                                                           | [`String`](https://stencila.ghost.io/docs/reference/schema/string) \| [`Number`](https://stencila.ghost.io/docs/reference/schema/number)                   | -                                                                  | [`schema:version`](https://schema.org/version)           | -                                |
| `identifiers`  | Any kind of identifier for the referenced work.                                                       | ([`PropertyValue`](https://stencila.ghost.io/docs/reference/schema/property-value) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string))* | -                                                                  | [`schema:identifier`](https://schema.org/identifier)     | `identifier`                     |
| `url`          | The URL of the referenced work.                                                                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                  | [`schema:url`](https://schema.org/url)                   | -                                |
| `text`         | Plain text representation of the referenced work.                                                     | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                  | [`schema:text`](https://schema.org/text)                 | -                                |

# Related

The `Reference` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `Reference` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                            | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                    |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |                                    |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |              |              |                                    |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | ⚠️ High loss |              | Encoded using implemented function |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | ⚠️ High loss |              |                                    |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | ⚠️ High loss |              |                                    |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | ⚠️ High loss |              |                                    |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | ⚠️ High loss |              |                                    |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                    |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                    |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |                                    |
| [OpenDocument Text](https://stencila.ghost.io/docs/reference/formats/odt)           | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |                                    |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                    |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |                                    |
| [CSV](https://stencila.ghost.io/docs/reference/formats/csv)                         |              |              |                                    |
| [TSV](https://stencila.ghost.io/docs/reference/formats/tsv)                         |              |              |                                    |
| [Apache Parquet](https://stencila.ghost.io/docs/reference/formats/parquet)          |              |              |                                    |
| [Apache Arrow](https://stencila.ghost.io/docs/reference/formats/arrow)              |              |              |                                    |
| [Microsoft Excel XLSX](https://stencila.ghost.io/docs/reference/formats/xlsx)       |              |              |                                    |
| [Microsoft Excel XLS](https://stencila.ghost.io/docs/reference/formats/xls)         |              |              |                                    |
| [OpenDocument Spreadsheet](https://stencila.ghost.io/docs/reference/formats/ods)    |              |              |                                    |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                    |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                    |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                    |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                    |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |                                    |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                    |

# Bindings

The `Reference` type is represented in:

- [JSON-LD](https://stencila.org/Reference.jsonld)
- [JSON Schema](https://stencila.org/Reference.schema.json)
- Python class [`Reference`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/reference.py)
- Rust struct [`Reference`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/reference.rs)
- TypeScript class [`Reference`](https://github.com/stencila/stencila/blob/main/ts/src/types/Reference.ts)

# Source

This documentation was generated from [`Reference.yaml`](https://github.com/stencila/stencila/blob/main/schema/Reference.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
