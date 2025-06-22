---
title: Suggestion Inline
description: Abstract base type for nodes that indicate a suggested change to inline content.
config:
  publish:
    ghost:
      type: post
      slug: suggestion-inline
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Edits
---

# Properties

The `SuggestionInline` type has these properties:

| Name                | Description                                                                                     | Type                                                                                    | Inherited from                                                             | `JSON-LD @id`                                | Aliases                                    |
| ------------------- | ----------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- | -------------------------------------------- | ------------------------------------------ |
| `id`                | The identifier for this item.                                                                   | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                      | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)         | [`schema:id`](https://schema.org/id)         | -                                          |
| `suggestionStatus`  | The status of the suggestion including whether it is the original, or is accepted, or rejected. | [`SuggestionStatus`](https://stencila.ghost.io/docs/reference/schema/suggestion-status) | [`Suggestion`](https://stencila.ghost.io/docs/reference/schema/suggestion) | `stencila:suggestionStatus`                  | `suggestion-status`, `suggestion_status`   |
| `authors`           | The authors of the suggestion                                                                   | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                     | [`Suggestion`](https://stencila.ghost.io/docs/reference/schema/suggestion) | [`schema:author`](https://schema.org/author) | `author`                                   |
| `provenance`        | A summary of the provenance of the content within the suggestion.                               | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)*  | [`Suggestion`](https://stencila.ghost.io/docs/reference/schema/suggestion) | `stencila:provenance`                        | -                                          |
| `executionDuration` | Time taken to generate the suggestion.                                                          | [`Duration`](https://stencila.ghost.io/docs/reference/schema/duration)                  | [`Suggestion`](https://stencila.ghost.io/docs/reference/schema/suggestion) | `stencila:executionDuration`                 | `execution-duration`, `execution_duration` |
| `executionEnded`    | The timestamp when the generation ended.                                                        | [`Timestamp`](https://stencila.ghost.io/docs/reference/schema/timestamp)                | [`Suggestion`](https://stencila.ghost.io/docs/reference/schema/suggestion) | `stencila:executionEnded`                    | `execution-ended`, `execution_ended`       |
| `feedback`          | Feedback on the suggestion                                                                      | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                      | [`Suggestion`](https://stencila.ghost.io/docs/reference/schema/suggestion) | `stencila:feedback`                          | -                                          |
| `content`           | The content that is suggested to be inserted, modified, replaced, or deleted.                   | [`Inline`](https://stencila.ghost.io/docs/reference/schema/inline)*                     | -                                                                          | `stencila:content`                           | -                                          |

# Related

The `SuggestionInline` type is related to these types:

- Parents: [`Suggestion`](https://stencila.ghost.io/docs/reference/schema/suggestion)
- Children: none

# Formats

The `SuggestionInline` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                               | Encoding     | Decoding   | Support                            | Notes |
| ------------------------------------------------------------------------------------ | ------------ | ---------- | ---------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)                | 🟢 No loss    |            |                                    |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                        | 🟢 No loss    |            |                                    |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                        |              |            |                                    |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                      | ⚠️ High loss |            | Encoded using implemented function |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)            | ⚠️ High loss |            |                                    |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)              | ⚠️ High loss |            |                                    |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)               | ⚠️ High loss |            |                                    |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)                | ⚠️ High loss |            |                                    |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                      | 🔷 Low loss   | 🔷 Low loss |                                    |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                      | 🔷 Low loss   | 🔷 Low loss |                                    |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                          | ⚠️ High loss |            |                                    |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                  | ⚠️ High loss |            |                                    |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                      | 🔷 Low loss   | 🔷 Low loss |                                    |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)         | 🔷 Low loss   | 🔷 Low loss |                                    |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)           |              |            |                                    |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)             | 🔷 Low loss   | 🔷 Low loss |                                    |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                          | 🔷 Low loss   | 🔷 Low loss |                                    |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                        | 🟢 No loss    | 🟢 No loss  |                                    |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)                | 🟢 No loss    | 🟢 No loss  |                                    |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                      | 🟢 No loss    | 🟢 No loss  |                                    |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                   | 🟢 No loss    | 🟢 No loss  |                                    |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                        | 🟢 No loss    | 🟢 No loss  |                                    |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)              | 🟢 No loss    | 🟢 No loss  |                                    |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                        | 🟢 No loss    | 🟢 No loss  |                                    |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)             | 🔷 Low loss   | 🔷 Low loss |                                    |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)               | 🔷 Low loss   | 🔷 Low loss |                                    |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)                | 🔷 Low loss   | 🔷 Low loss |                                    |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                          | ⚠️ High loss |            |                                    |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)              |              |            |                                    |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)          |              |            |                                    |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoap) |              | 🔷 Low loss |                                    |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                      | 🔷 Low loss   |            |                                    |

# Bindings

The `SuggestionInline` type is represented in:

- [JSON-LD](https://stencila.org/SuggestionInline.jsonld)
- [JSON Schema](https://stencila.org/SuggestionInline.schema.json)
- Python class [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/suggestion_inline.py)
- Rust struct [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion_inline.rs)
- TypeScript class [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/SuggestionInline.ts)

# Source

This documentation was generated from [`SuggestionInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/SuggestionInline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
