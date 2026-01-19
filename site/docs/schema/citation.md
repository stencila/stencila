---
title: Citation
description: A reference to a `CreativeWork` that is cited in another `CreativeWork`.
---

A `Citation` node is used within a [`CreativeWork`](./CreativeWork), usually an
[`Article`](./Article), to refer to an other `CreativeWork`.
Often a `Citation` will be associated with other citations, in a `CitationGroup`.


# Properties

The `Citation` type has these properties:

| Name                  | Description                                                                                           | Type                                                 | Inherited from          | `JSON-LD @id`                                        | Aliases                                                                                                            |
| --------------------- | ----------------------------------------------------------------------------------------------------- | ---------------------------------------------------- | ----------------------- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                                                                         | [`String`](./string.md)                              | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)                 | -                                                                                                                  |
| `target`              | The target of the citation (URL or reference ID).                                                     | [`String`](./string.md)                              | -                       | [`schema:target`](https://schema.org/target)         | -                                                                                                                  |
| `compilationMessages` | Messages generated while resolving the target if the citation.                                        | [`CompilationMessage`](./compilation-message.md)*    | -                       | `stencila:compilationMessages`                       | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |
| `cites`               | The `Reference` being cited, resolved from the `target`.                                              | [`Reference`](./reference.md)                        | -                       | `stencila:cites`                                     | -                                                                                                                  |
| `citationMode`        | Determines how the citation is shown within the surrounding text.                                     | [`CitationMode`](./citation-mode.md)                 | -                       | `stencila:citationMode`                              | `citation-mode`, `citation_mode`                                                                                   |
| `citationIntent`      | The type/s of the citation, both factually and rhetorically.                                          | [`CitationIntent`](./citation-intent.md)*            | -                       | `stencila:citationIntent`                            | `citation-intent`, `citation_intent`                                                                               |
| `content`             | A rendering of the citation using the citation style of the document.                                 | [`Inline`](./inline.md)*                             | -                       | `stencila:content`                                   | -                                                                                                                  |
| `pageStart`           | The page on which the work starts; for example "135" or "xiii".                                       | [`Integer`](./integer.md) \| [`String`](./string.md) | -                       | [`schema:pageStart`](https://schema.org/pageStart)   | `page-start`, `page_start`                                                                                         |
| `pageEnd`             | The page on which the work ends; for example "138" or "xvi".                                          | [`Integer`](./integer.md) \| [`String`](./string.md) | -                       | [`schema:pageEnd`](https://schema.org/pageEnd)       | `page-end`, `page_end`                                                                                             |
| `pagination`          | Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55". | [`String`](./string.md)                              | -                       | [`schema:pagination`](https://schema.org/pagination) | -                                                                                                                  |
| `citationPrefix`      | Text to show before the citation.                                                                     | [`String`](./string.md)                              | -                       | `stencila:citationPrefix`                            | `citation-prefix`, `citation_prefix`                                                                               |
| `citationSuffix`      | Text to show after the citation.                                                                      | [`String`](./string.md)                              | -                       | `stencila:citationSuffix`                            | `citation-suffix`, `citation_suffix`                                                                               |

# Related

The `Citation` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `Citation` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                            | Notes |
| ------------------------------------------------ | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                    |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |                                    |
| [JATS](../formats/jats.md)                       |              |              | Encoded using special function     |
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

The `Citation` type is represented in:

- [JSON-LD](https://stencila.org/Citation.jsonld)
- [JSON Schema](https://stencila.org/Citation.schema.json)
- Python class [`Citation`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/citation.py)
- Rust struct [`Citation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/citation.rs)
- TypeScript class [`Citation`](https://github.com/stencila/stencila/blob/main/ts/src/types/Citation.ts)

# Source

This documentation was generated from [`Citation.yaml`](https://github.com/stencila/stencila/blob/main/schema/Citation.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
