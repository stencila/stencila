---
title: Timestamp Validator
description: A validator specifying the constraints on a timestamp.
config:
  publish:
    ghost:
      type: post
      slug: timestamp-validator
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Data
---

# Properties

The `TimestampValidator` type has these properties:

| Name        | Description                                 | Type                                                                     | Inherited from                                                     | `JSON-LD @id`                        | Aliases                                                          |
| ----------- | ------------------------------------------- | ------------------------------------------------------------------------ | ------------------------------------------------------------------ | ------------------------------------ | ---------------------------------------------------------------- |
| `id`        | The identifier for this item.               | [`String`](https://stencila.ghost.io/docs/reference/schema/string)       | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -                                                                |
| `timeUnits` | The time units that the timestamp can have. | [`TimeUnit`](https://stencila.ghost.io/docs/reference/schema/time-unit)* | -                                                                  | `stencila:timeUnits`                 | `time-units`, `time_units`, `timeUnit`, `time-unit`, `time_unit` |
| `minimum`   | The inclusive lower limit for a timestamp.  | [`Timestamp`](https://stencila.ghost.io/docs/reference/schema/timestamp) | -                                                                  | `stencila:minimum`                   | -                                                                |
| `maximum`   | The inclusive upper limit for a timestamp.  | [`Timestamp`](https://stencila.ghost.io/docs/reference/schema/timestamp) | -                                                                  | `stencila:maximum`                   | -                                                                |

# Related

The `TimestampValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `TimestampValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                          | 🔷 Low loss   |            |                                    |
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
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)              |              |            |                                    |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)          |              |            |                                    |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoap) |              | 🔷 Low loss |                                    |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                      | 🔷 Low loss   |            |                                    |

# Bindings

The `TimestampValidator` type is represented in:

- [JSON-LD](https://stencila.org/TimestampValidator.jsonld)
- [JSON Schema](https://stencila.org/TimestampValidator.schema.json)
- Python class [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/timestamp_validator.py)
- Rust struct [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/timestamp_validator.rs)
- TypeScript class [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/TimestampValidator.ts)

# Source

This documentation was generated from [`TimestampValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/TimestampValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
