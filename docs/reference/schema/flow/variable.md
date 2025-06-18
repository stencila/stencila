---
title: Variable
description: A variable representing a name / value pair.
config:
  publish:
    ghost:
      type: post
      slug: variable
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Flow
---

This type is marked as unstable and is subject to change.

# Properties

The `Variable` type has these properties:

| Name                  | Description                                                                           | Type                                                               | Inherited from                                                     | `JSON-LD @id`                                                          | Aliases                                        |
| --------------------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------ | ---------------------------------------------------------------------- | ---------------------------------------------- |
| `id`                  | The identifier for this item.                                                         | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                                   | -                                              |
| `name`                | The name of the variable.                                                             | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | -                                                                  | [`schema:name`](https://schema.org/name)                               | -                                              |
| `programmingLanguage` | The programming language that the variable is defined in e.g. Python, JSON.           | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | -                                                                  | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | `programming-language`, `programming_language` |
| `nativeType`          | The native type of the variable e.g. `float`, `datetime.datetime`, `pandas.DataFrame` | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | -                                                                  | `stencila:nativeType`                                                  | `native-type`, `native_type`                   |
| `nodeType`            | The Stencila node type of the variable e.g. `Number`, `DateTime`, `Datatable`.        | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | -                                                                  | `stencila:nodeType`                                                    | `node-type`, `node_type`                       |
| `value`               | The value of the variable.                                                            | [`Node`](https://stencila.ghost.io/docs/reference/schema/node)     | -                                                                  | [`schema:value`](https://schema.org/value)                             | -                                              |
| `hint`                | A hint to the value and/or structure of the variable.                                 | [`Hint`](https://stencila.ghost.io/docs/reference/schema/hint)     | -                                                                  | `stencila:hint`                                                        | -                                              |
| `nativeHint`          | A textual hint to the value and/or structure of the variable.                         | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | -                                                                  | `stencila:nativeHint`                                                  | `native-hint`, `native_hint`                   |

# Related

The `Variable` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `Variable` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                      | 🔷 Low loss   | 🔷 Low loss |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                          | ⚠️ High loss |            |         |
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
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                          | ⚠️ High loss |            |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)              |              |            |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)          |              |            |         |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoap) |              | 🔷 Low loss |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                      | 🔷 Low loss   |            |         |

# Bindings

The `Variable` type is represented in:

- [JSON-LD](https://stencila.org/Variable.jsonld)
- [JSON Schema](https://stencila.org/Variable.schema.json)
- Python class [`Variable`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/variable.py)
- Rust struct [`Variable`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/variable.rs)
- TypeScript class [`Variable`](https://github.com/stencila/stencila/blob/main/ts/src/types/Variable.ts)

# Source

This documentation was generated from [`Variable.yaml`](https://github.com/stencila/stencila/blob/main/schema/Variable.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
