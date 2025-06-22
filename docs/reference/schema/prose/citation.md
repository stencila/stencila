---
title: Citation
description: A reference to a `CreativeWork` that is cited in another `CreativeWork`.
config:
  publish:
    ghost:
      type: post
      slug: citation
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

A `Citation` node is used within a [`CreativeWork`](./CreativeWork), usually an
[`Article`](./Article), to refer to an other `CreativeWork`.
Often a `Citation` will be associated with other citations, in a `CitationGroup`.


# Properties

The `Citation` type has these properties:

| Name                  | Description                                                                                           | Type                                                                                                                                       | Inherited from                                                     | `JSON-LD @id`                                        | Aliases                                                                                                            |
| --------------------- | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------ | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                                                                         | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                         | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                 | -                                                                                                                  |
| `target`              | The target of the citation (URL or reference ID).                                                     | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                         | -                                                                  | [`schema:target`](https://schema.org/target)         | -                                                                                                                  |
| `compilationMessages` | Messages generated while resolving the target if the citation.                                        | [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message)*                                               | -                                                                  | `stencila:compilationMessages`                       | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |
| `cites`               | The `Reference` being cited, resolved from the `target`.                                              | [`Reference`](https://stencila.ghost.io/docs/reference/schema/reference)                                                                   | -                                                                  | `stencila:cites`                                     | -                                                                                                                  |
| `citationMode`        | Determines how the citation is shown within the surrounding text.                                     | [`CitationMode`](https://stencila.ghost.io/docs/reference/schema/citation-mode)                                                            | -                                                                  | `stencila:citationMode`                              | `citation-mode`, `citation_mode`                                                                                   |
| `citationIntent`      | The type/s of the citation, both factually and rhetorically.                                          | [`CitationIntent`](https://stencila.ghost.io/docs/reference/schema/citation-intent)*                                                       | -                                                                  | `stencila:citationIntent`                            | `citation-intent`, `citation_intent`                                                                               |
| `content`             | Optional structured content/text of this citation.                                                    | [`Inline`](https://stencila.ghost.io/docs/reference/schema/inline)*                                                                        | -                                                                  | `stencila:content`                                   | -                                                                                                                  |
| `pageStart`           | The page on which the work starts; for example "135" or "xiii".                                       | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string) | -                                                                  | [`schema:pageStart`](https://schema.org/pageStart)   | `page-start`, `page_start`                                                                                         |
| `pageEnd`             | The page on which the work ends; for example "138" or "xvi".                                          | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string) | -                                                                  | [`schema:pageEnd`](https://schema.org/pageEnd)       | `page-end`, `page_end`                                                                                             |
| `pagination`          | Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55". | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                         | -                                                                  | [`schema:pagination`](https://schema.org/pagination) | -                                                                                                                  |
| `citationPrefix`      | Text to show before the citation.                                                                     | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                         | -                                                                  | `stencila:citationPrefix`                            | `citation-prefix`, `citation_prefix`                                                                               |
| `citationSuffix`      | Text to show after the citation.                                                                      | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                         | -                                                                  | `stencila:citationSuffix`                            | `citation-suffix`, `citation_suffix`                                                                               |

# Related

The `Citation` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `Citation` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                               | Encoding     | Decoding   | Support                            | Notes |
| ------------------------------------------------------------------------------------ | ------------ | ---------- | ---------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)                | 🟢 No loss    |            |                                    |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                        | 🟢 No loss    |            |                                    |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                        |              |            | Encoded using special function     |
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

The `Citation` type is represented in:

- [JSON-LD](https://stencila.org/Citation.jsonld)
- [JSON Schema](https://stencila.org/Citation.schema.json)
- Python class [`Citation`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/citation.py)
- Rust struct [`Citation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/citation.rs)
- TypeScript class [`Citation`](https://github.com/stencila/stencila/blob/main/ts/src/types/Citation.ts)

# Source

This documentation was generated from [`Citation.yaml`](https://github.com/stencila/stencila/blob/main/schema/Citation.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
