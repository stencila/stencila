---
title: Code Location
description: The location within some source code.
config:
  publish:
    ghost:
      type: post
      slug: code-location
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Flow
---

# Properties

The `CodeLocation` type has these properties:

| Name          | Description                                                        | Type                                                                                  | Inherited from                                                     | `JSON-LD @id`                        | Aliases                        |
| ------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------ | ------------------------------ |
| `id`          | The identifier for this item.                                      | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                    | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -                              |
| `source`      | The source of the code, a file path, label or URL.                 | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                    | -                                                                  | `stencila:source`                    | -                              |
| `startLine`   | The 0-based index if the first line on which the error occurred.   | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:startLine`                 | `start-line`, `start_line`     |
| `startColumn` | The 0-based index if the first column on which the error occurred. | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:startColumn`               | `start-column`, `start_column` |
| `endLine`     | The 0-based index if the last line on which the error occurred.    | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:endLine`                   | `end-line`, `end_line`         |
| `endColumn`   | The 0-based index if the last column on which the error occurred.  | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | `stencila:endColumn`                 | `end-column`, `end_column`     |

# Related

The `CodeLocation` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `CodeLocation` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                               | Encoding     | Decoding   | Support | Notes |
| ------------------------------------------------------------------------------------ | ------------ | ---------- | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)                | 🟢 No loss    |            |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                        | 🔷 Low loss   |            |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                        |              |            |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                      | ⚠️ High loss |            |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)            | ⚠️ High loss |            |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)              | ⚠️ High loss |            |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)               | ⚠️ High loss |            |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)                | ⚠️ High loss |            |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                      | 🔷 Low loss   | 🔷 Low loss |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                          | 🔷 Low loss   |            |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                  | ⚠️ High loss |            |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                      | 🔷 Low loss   | 🔷 Low loss |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)         | 🔷 Low loss   | 🔷 Low loss |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)             | 🔷 Low loss   | 🔷 Low loss |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                          | 🔷 Low loss   | 🔷 Low loss |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                        | 🟢 No loss    | 🟢 No loss  |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)                | 🟢 No loss    | 🟢 No loss  |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                      | 🟢 No loss    | 🟢 No loss  |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                   | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                        | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)              | 🟢 No loss    | 🟢 No loss  |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                        | 🟢 No loss    | 🟢 No loss  |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)             | 🔷 Low loss   | 🔷 Low loss |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)               | 🔷 Low loss   | 🔷 Low loss |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)                | 🔷 Low loss   | 🔷 Low loss |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)              |              |            |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)          |              |            |         |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoap) |              | 🔷 Low loss |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                      | 🔷 Low loss   |            |         |

# Bindings

The `CodeLocation` type is represented in:

- [JSON-LD](https://stencila.org/CodeLocation.jsonld)
- [JSON Schema](https://stencila.org/CodeLocation.schema.json)
- Python class [`CodeLocation`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/code_location.py)
- Rust struct [`CodeLocation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_location.rs)
- TypeScript class [`CodeLocation`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeLocation.ts)

# Source

This documentation was generated from [`CodeLocation.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeLocation.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
