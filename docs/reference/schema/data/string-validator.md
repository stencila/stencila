---
title: String Validator
description: A schema specifying constraints on a string node.
config:
  publish:
    ghost:
      type: post
      slug: string-validator
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Data
---

A node will be valid against the schema if it is a string that
meets the schemas `minLength`, `maxLength` and `pattern` properties.
Analogous to the JSON Schema `string` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).


# Properties

The `StringValidator` type has these properties:

| Name        | Description                                         | Type                                                                 | Inherited from                                                     | `JSON-LD @id`                        | Aliases                    |
| ----------- | --------------------------------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------ | -------------------------- |
| `id`        | The identifier for this item.                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)   | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -                          |
| `minLength` | The minimum length for a string node.               | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer) | -                                                                  | `stencila:minLength`                 | `min-length`, `min_length` |
| `maxLength` | The maximum length for a string node.               | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer) | -                                                                  | `stencila:maxLength`                 | `max-length`, `max_length` |
| `pattern`   | A regular expression that a string node must match. | [`String`](https://stencila.ghost.io/docs/reference/schema/string)   | -                                                                  | `stencila:pattern`                   | -                          |

# Related

The `StringValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `StringValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                               | Encoding     | Decoding   | Support                            | Notes |
| ------------------------------------------------------------------------------------ | ------------ | ---------- | ---------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)                | 🟢 No loss    |            |                                    |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                        | 🔷 Low loss   |            |                                    |
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

The `StringValidator` type is represented in:

- [JSON-LD](https://stencila.org/StringValidator.jsonld)
- [JSON Schema](https://stencila.org/StringValidator.schema.json)
- Python class [`StringValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/string_validator.py)
- Rust struct [`StringValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/string_validator.rs)
- TypeScript class [`StringValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/StringValidator.ts)

# Source

This documentation was generated from [`StringValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/StringValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
