---
title: Suggestion Inline
description: Abstract base type for nodes that indicate a suggested change to inline content.
---

# Properties

The `SuggestionInline` type has these properties:

| Name                | Description                                                                                     | Type                                         | Inherited from                  | `JSON-LD @id`                                | Aliases                                    |
| ------------------- | ----------------------------------------------------------------------------------------------- | -------------------------------------------- | ------------------------------- | -------------------------------------------- | ------------------------------------------ |
| `id`                | The identifier for this item.                                                                   | [`String`](./string.md)                      | [`Entity`](./entity.md)         | [`schema:id`](https://schema.org/id)         | -                                          |
| `suggestionStatus`  | The status of the suggestion including whether it is the original, or is accepted, or rejected. | [`SuggestionStatus`](./suggestion-status.md) | [`Suggestion`](./suggestion.md) | `stencila:suggestionStatus`                  | `suggestion-status`, `suggestion_status`   |
| `authors`           | The authors of the suggestion                                                                   | [`Author`](./author.md)*                     | [`Suggestion`](./suggestion.md) | [`schema:author`](https://schema.org/author) | `author`                                   |
| `provenance`        | A summary of the provenance of the content within the suggestion.                               | [`ProvenanceCount`](./provenance-count.md)*  | [`Suggestion`](./suggestion.md) | `stencila:provenance`                        | -                                          |
| `executionDuration` | Time taken to generate the suggestion.                                                          | [`Duration`](./duration.md)                  | [`Suggestion`](./suggestion.md) | `stencila:executionDuration`                 | `execution-duration`, `execution_duration` |
| `executionEnded`    | The timestamp when the generation ended.                                                        | [`Timestamp`](./timestamp.md)                | [`Suggestion`](./suggestion.md) | `stencila:executionEnded`                    | `execution-ended`, `execution_ended`       |
| `feedback`          | Feedback on the suggestion                                                                      | [`String`](./string.md)                      | [`Suggestion`](./suggestion.md) | `stencila:feedback`                          | -                                          |
| `content`           | The content that is suggested to be inserted, modified, replaced, or deleted.                   | [`Inline`](./inline.md)*                     | -                               | `stencila:content`                           | -                                          |

# Related

The `SuggestionInline` type is related to these types:

- Parents: [`Suggestion`](./suggestion.md)
- Children: none

# Formats

The `SuggestionInline` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `SuggestionInline` type is represented in:

- [JSON-LD](https://stencila.org/SuggestionInline.jsonld)
- [JSON Schema](https://stencila.org/SuggestionInline.schema.json)
- Python class [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/suggestion_inline.py)
- Rust struct [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion_inline.rs)
- TypeScript class [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/SuggestionInline.ts)

# Source

This documentation was generated from [`SuggestionInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/SuggestionInline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
