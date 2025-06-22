---
title: Island
description: An island of content in a document.
config:
  publish:
    ghost:
      type: post
      slug: island
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Other
---

# Properties

The `Island` type has these properties:

| Name                 | Description                                        | Type                                                                      | Inherited from                                                     | `JSON-LD @id`                        | Aliases                                      |
| -------------------- | -------------------------------------------------- | ------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------ | -------------------------------------------- |
| `id`                 | The identifier for this item.                      | [`String`](https://stencila.ghost.io/docs/reference/schema/string)        | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -                                            |
| `content`            | The content within the section.                    | [`Block`](https://stencila.ghost.io/docs/reference/schema/block)*         | -                                                                  | `stencila:content`                   | -                                            |
| `isAutomatic`        | Whether the island is automatically generated.     | [`Boolean`](https://stencila.ghost.io/docs/reference/schema/boolean)      | -                                                                  | `stencila:isAutomatic`               | `is-automatic`, `is_automatic`               |
| `labelType`          | The type of the label for the island.              | [`LabelType`](https://stencila.ghost.io/docs/reference/schema/label-type) | -                                                                  | `stencila:labelType`                 | `label-type`, `label_type`                   |
| `label`              | A short label for the chunk.                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)        | -                                                                  | `stencila:label`                     | -                                            |
| `labelAutomatically` | Whether the label should be automatically updated. | [`Boolean`](https://stencila.ghost.io/docs/reference/schema/boolean)      | -                                                                  | `stencila:labelAutomatically`        | `label-automatically`, `label_automatically` |
| `style`              | The style to apply to the island.                  | [`String`](https://stencila.ghost.io/docs/reference/schema/string)        | -                                                                  | `stencila:style`                     | -                                            |

# Related

The `Island` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `Island` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                               | Encoding     | Decoding   | Support | Notes |
| ------------------------------------------------------------------------------------ | ------------ | ---------- | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)                | 🟢 No loss    |            |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                        | 🟢 No loss    |            |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                        |              |            |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                      | ⚠️ High loss |            |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)            | ⚠️ High loss |            |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)              | ⚠️ High loss |            |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)               | ⚠️ High loss |            |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)                | ⚠️ High loss |            |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                      | 🔷 Low loss   | 🔷 Low loss |         |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                      | 🔷 Low loss   | 🔷 Low loss |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                          | ⚠️ High loss |            |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                  | ⚠️ High loss |            |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                      | 🔷 Low loss   | 🔷 Low loss |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)         | 🔷 Low loss   | 🔷 Low loss |         |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)           |              |            |         |
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
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                          | ⚠️ High loss |            |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)              |              |            |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)          |              |            |         |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoap) |              | 🔷 Low loss |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                      | 🔷 Low loss   |            |         |

# Bindings

The `Island` type is represented in:

- [JSON-LD](https://stencila.org/Island.jsonld)
- [JSON Schema](https://stencila.org/Island.schema.json)
- Python class [`Island`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/island.py)
- Rust struct [`Island`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/island.rs)
- TypeScript class [`Island`](https://github.com/stencila/stencila/blob/main/ts/src/types/Island.ts)

# Source

This documentation was generated from [`Island.yaml`](https://github.com/stencila/stencila/blob/main/schema/Island.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
