---
title: Tuple Validator
description: A validator specifying constraints on an array of heterogeneous items.
config:
  publish:
    ghost:
      type: post
      slug: tuple-validator
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Data
---

# Properties

The `TupleValidator` type has these properties:

| Name    | Description                                                                             | Type                                                                      | Inherited from                                                     | `JSON-LD @id`                                                  | Aliases |
| ------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------------------------- | ------- |
| `id`    | The identifier for this item.                                                           | [`String`](https://stencila.ghost.io/docs/reference/schema/string)        | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                           | -       |
| `items` | An array of validators specifying the constraints on each successive item in the array. | [`Validator`](https://stencila.ghost.io/docs/reference/schema/validator)* | -                                                                  | [`schema:itemListElement`](https://schema.org/itemListElement) | `item`  |

# Related

The `TupleValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `TupleValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `TupleValidator` type is represented in:

- [JSON-LD](https://stencila.org/TupleValidator.jsonld)
- [JSON Schema](https://stencila.org/TupleValidator.schema.json)
- Python class [`TupleValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/tuple_validator.py)
- Rust struct [`TupleValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/tuple_validator.rs)
- TypeScript class [`TupleValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/TupleValidator.ts)

# Source

This documentation was generated from [`TupleValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/TupleValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
