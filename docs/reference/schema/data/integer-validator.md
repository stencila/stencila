---
title: Integer Validator
description: A validator specifying the constraints on an integer node.
config:
  publish:
    ghost:
      type: post
      slug: integer-validator
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Data
---

A node will be valid if it is a number with no fractional part and meets any additional constraints,
such as `multipleOf`, specified in the validator.
Analogous to the JSON Schema `integer` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).


# Properties

The `IntegerValidator` type has these properties:

| Name               | Description                                         | Type                                                               | Inherited from                                                                        | `JSON-LD @id`                        | Aliases                                  |
| ------------------ | --------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------- | ------------------------------------ | ---------------------------------------- |
| `id`               | The identifier for this item.                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)                    | [`schema:id`](https://schema.org/id) | -                                        |
| `minimum`          | The inclusive lower limit for a numeric node.       | [`Number`](https://stencila.ghost.io/docs/reference/schema/number) | [`NumberValidator`](https://stencila.ghost.io/docs/reference/schema/number-validator) | `stencila:minimum`                   | -                                        |
| `exclusiveMinimum` | The exclusive lower limit for a numeric node.       | [`Number`](https://stencila.ghost.io/docs/reference/schema/number) | [`NumberValidator`](https://stencila.ghost.io/docs/reference/schema/number-validator) | `stencila:exclusiveMinimum`          | `exclusive-minimum`, `exclusive_minimum` |
| `maximum`          | The inclusive upper limit for a numeric node.       | [`Number`](https://stencila.ghost.io/docs/reference/schema/number) | [`NumberValidator`](https://stencila.ghost.io/docs/reference/schema/number-validator) | `stencila:maximum`                   | -                                        |
| `exclusiveMaximum` | The exclusive upper limit for a numeric node.       | [`Number`](https://stencila.ghost.io/docs/reference/schema/number) | [`NumberValidator`](https://stencila.ghost.io/docs/reference/schema/number-validator) | `stencila:exclusiveMaximum`          | `exclusive-maximum`, `exclusive_maximum` |
| `multipleOf`       | A number that a numeric node must be a multiple of. | [`Number`](https://stencila.ghost.io/docs/reference/schema/number) | [`NumberValidator`](https://stencila.ghost.io/docs/reference/schema/number-validator) | `stencila:multipleOf`                | `multiple-of`, `multiple_of`             |

# Related

The `IntegerValidator` type is related to these types:

- Parents: [`NumberValidator`](https://stencila.ghost.io/docs/reference/schema/number-validator)
- Children: none

# Formats

The `IntegerValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                            | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                    |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |                                    |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |              |              |                                    |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | ⚠️ High loss |              | Encoded using implemented function |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | ⚠️ High loss |              |                                    |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | ⚠️ High loss |              |                                    |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | ⚠️ High loss |              |                                    |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | ⚠️ High loss |              |                                    |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                    |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                    |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |                                    |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |                                    |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                    |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |                                    |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                    |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                    |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                    |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                    |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |                                    |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                    |

# Bindings

The `IntegerValidator` type is represented in:

- [JSON-LD](https://stencila.org/IntegerValidator.jsonld)
- [JSON Schema](https://stencila.org/IntegerValidator.schema.json)
- Python class [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/integer_validator.py)
- Rust struct [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/integer_validator.rs)
- TypeScript class [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/IntegerValidator.ts)

# Source

This documentation was generated from [`IntegerValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/IntegerValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
