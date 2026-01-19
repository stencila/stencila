---
title: Supplement
description: A supplementary `CreativeWork` that supports this work but is not considered part of its main content.
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

| Name                  | Description                                                                 | Type                                                | Inherited from          | `JSON-LD @id`                                  | Aliases                                                                                                            |
| --------------------- | --------------------------------------------------------------------------- | --------------------------------------------------- | ----------------------- | ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                                               | [`String`](./string.md)                             | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)           | -                                                                                                                  |
| `workType`            | The `CreativeWork` type of the supplement.                                  | [`CreativeWorkType`](./creative-work-type.md)       | -                       | `stencila:workType`                            | `work-type`, `work_type`                                                                                           |
| `label`               | A short identifier or title for the supplement (e.g., "S1").                | [`String`](./string.md)                             | -                       | `stencila:label`                               | -                                                                                                                  |
| `labelAutomatically`  | Whether the supplement label should be automatically generated and updated. | [`Boolean`](./boolean.md)                           | -                       | `stencila:labelAutomatically`                  | `label-automatically`, `label_automatically`                                                                       |
| `caption`             | A brief caption or description for the supplement.                          | [`Block`](./block.md)*                              | -                       | [`schema:caption`](https://schema.org/caption) | -                                                                                                                  |
| `target`              | A reference to the supplement.                                              | [`String`](./string.md)                             | -                       | [`schema:target`](https://schema.org/target)   | -                                                                                                                  |
| `compilationMessages` | Any messages generated while embedding the supplement.                      | [`CompilationMessage`](./compilation-message.md)*   | -                       | `stencila:compilationMessages`                 | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |
| `work`                | The `CreativeWork` that constitutes the supplement.                         | [`CreativeWorkVariant`](./creative-work-variant.md) | -                       | `stencila:work`                                | -                                                                                                                  |

# Related

The `Supplement` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `Supplement` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                                                                | Notes |
| ------------------------------------------------ | ------------ | ------------ | -------------------------------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                                                        |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |                                                                                                                                        |
| [JATS](../formats/jats.md)                       |              |              | Encoded as [`<supplementary-material>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/supplementary-material.html) |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              | Encoded using implemented function                                                                                                     |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |                                                                                                                                        |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |                                                                                                                                        |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |                                                                                                                                        |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |                                                                                                                                        |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                                                        |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                                                                        |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                                        |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                                        |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                                                        |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                                        |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                                        |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                                        |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                                        |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                        |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                                                                        |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                                                                        |
| [CSV](../formats/csv.md)                         |              |              |                                                                                                                                        |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                                                                        |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                                                                        |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                                                                        |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                                                                        |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                                                                        |
| [Directory](../formats/directory.md)             |              |              |                                                                                                                                        |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                                                                        |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                                                                        |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                                                                        |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                                                                        |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                                                                        |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                                                                        |

# Bindings

The `Supplement` type is represented in:

- [JSON-LD](https://stencila.org/Supplement.jsonld)
- [JSON Schema](https://stencila.org/Supplement.schema.json)
- Python class [`Supplement`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/supplement.py)
- Rust struct [`Supplement`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/supplement.rs)
- TypeScript class [`Supplement`](https://github.com/stencila/stencila/blob/main/ts/src/types/Supplement.ts)

# Source

This documentation was generated from [`Supplement.yaml`](https://github.com/stencila/stencila/blob/main/schema/Supplement.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
