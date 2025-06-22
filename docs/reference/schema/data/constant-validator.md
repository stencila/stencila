---
title: Constant Validator
description: A validator specifying a constant value that a node must have.
config:
  publish:
    ghost:
      type: post
      slug: constant-validator
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Data
---

A node will be valid against this validator if it is equal to the
`value` property. Analogous to the JSON Schema [`const`](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.3) keyword.


# Properties

The `ConstantValidator` type has these properties:

| Name    | Description                        | Type                                                               | Inherited from                                                     | `JSON-LD @id`                              | Aliases |
| ------- | ---------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------ | ------------------------------------------ | ------- |
| `id`    | The identifier for this item.      | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)       | -       |
| `value` | The value that the node must have. | [`Node`](https://stencila.ghost.io/docs/reference/schema/node)     | -                                                                  | [`schema:value`](https://schema.org/value) | -       |

# Related

The `ConstantValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `ConstantValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `ConstantValidator` type is represented in:

- [JSON-LD](https://stencila.org/ConstantValidator.jsonld)
- [JSON Schema](https://stencila.org/ConstantValidator.schema.json)
- Python class [`ConstantValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/constant_validator.py)
- Rust struct [`ConstantValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/constant_validator.rs)
- TypeScript class [`ConstantValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConstantValidator.ts)

# Source

This documentation was generated from [`ConstantValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConstantValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
