---
title: Supplement
description: A supplementary `CreativeWork` that supports this work but is not considered part of its main content.
config:
  publish:
    ghost:
      type: post
      slug: supplement
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Works
---

Corresponds to the JATS `<supplementary-material>` element 
(https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/supplementary-material.html).

As in JATS, this is a `Block` content type so that supplementary material 
can be positioned close to the content it relates to (e.g., within a figure caption) 
rather than only at the end of an article. Nevertheless, many articles 
will include a dedicated "Supplementary Materials" section composed of a `Heading` 
followed by one or more `Supplement` blocks.


# Properties

The `Supplement` type has these properties:

| Name                  | Description                                                                 | Type                                                                                           | Inherited from                                                     | `JSON-LD @id`                                  | Aliases                                                                                                            |
| --------------------- | --------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                                               | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)           | -                                                                                                                  |
| `workType`            | The `CreativeWork` type of the supplement.                                  | [`CreativeWorkType`](https://stencila.ghost.io/docs/reference/schema/creative-work-type)       | -                                                                  | `stencila:workType`                            | `work-type`, `work_type`                                                                                           |
| `label`               | A short identifier or title for the supplement (e.g., "S1").                | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | -                                                                  | `stencila:label`                               | -                                                                                                                  |
| `labelAutomatically`  | Whether the supplement label should be automatically generated and updated. | [`Boolean`](https://stencila.ghost.io/docs/reference/schema/boolean)                           | -                                                                  | `stencila:labelAutomatically`                  | `label-automatically`, `label_automatically`                                                                       |
| `caption`             | A brief caption or description for the supplement.                          | [`Block`](https://stencila.ghost.io/docs/reference/schema/block)*                              | -                                                                  | [`schema:caption`](https://schema.org/caption) | -                                                                                                                  |
| `target`              | A reference to the supplement.                                              | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | -                                                                  | [`schema:target`](https://schema.org/target)   | -                                                                                                                  |
| `compilationMessages` | Any messages generated while embedding the supplement.                      | [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message)*   | -                                                                  | `stencila:compilationMessages`                 | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |
| `work`                | The `CreativeWork` that constitutes the supplement.                         | [`CreativeWorkVariant`](https://stencila.ghost.io/docs/reference/schema/creative-work-variant) | -                                                                  | `stencila:work`                                | -                                                                                                                  |

# Related

The `Supplement` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `Supplement` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                                                                                                                                | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | -------------------------------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                                                                                                                        |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |                                                                                                                                        |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |              |              | Encoded as [`<supplementary-material>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/supplementary-material.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | ⚠️ High loss |              | Encoded using implemented function                                                                                                     |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | ⚠️ High loss |              |                                                                                                                                        |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | ⚠️ High loss |              |                                                                                                                                        |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | ⚠️ High loss |              |                                                                                                                                        |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | ⚠️ High loss |              |                                                                                                                                        |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                                                        |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                                                                                                                        |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [Microsoft Word](https://stencila.ghost.io/docs/reference/formats/docx)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [OpenDocument Text](https://stencila.ghost.io/docs/reference/formats/odt)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                                        |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                                        |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                                                        |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                                        |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                                        |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/czst)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                                        |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                                        |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                                                                                                                        |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |                                                                                                                                        |
| [CSV](https://stencila.ghost.io/docs/reference/formats/csv)                         |              |              |                                                                                                                                        |
| [TSV](https://stencila.ghost.io/docs/reference/formats/tsv)                         |              |              |                                                                                                                                        |
| [Microsoft Excel](https://stencila.ghost.io/docs/reference/formats/xlsx)            |              |              |                                                                                                                                        |
| [Microsoft Excel (XLS)](https://stencila.ghost.io/docs/reference/formats/xls)       |              |              |                                                                                                                                        |
| [OpenDocument Spreadsheet](https://stencila.ghost.io/docs/reference/formats/ods)    |              |              |                                                                                                                                        |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                                                                                                                        |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                                                                                                                        |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                                                                                                                        |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                                                                                                                        |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              |              |                                                                                                                                        |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                                                                                                                        |

# Bindings

The `Supplement` type is represented in:

- [JSON-LD](https://stencila.org/Supplement.jsonld)
- [JSON Schema](https://stencila.org/Supplement.schema.json)
- Python class [`Supplement`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/supplement.py)
- Rust struct [`Supplement`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/supplement.rs)
- TypeScript class [`Supplement`](https://github.com/stencila/stencila/blob/main/ts/src/types/Supplement.ts)

# Source

This documentation was generated from [`Supplement.yaml`](https://github.com/stencila/stencila/blob/main/schema/Supplement.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
