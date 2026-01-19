---
title: Integer Validator
description: A validator specifying the constraints on an integer node.
---

A node will be valid if it is a number with no fractional part and meets any additional constraints,
such as `multipleOf`, specified in the validator.
Analogous to the JSON Schema `integer` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).


# Properties

The `IntegerValidator` type has these properties:

| Name               | Description                                         | Type                    | Inherited from                             | `JSON-LD @id`                        | Aliases                                  |
| ------------------ | --------------------------------------------------- | ----------------------- | ------------------------------------------ | ------------------------------------ | ---------------------------------------- |
| `id`               | The identifier for this item.                       | [`String`](./string.md) | [`Entity`](./entity.md)                    | [`schema:id`](https://schema.org/id) | -                                        |
| `minimum`          | The inclusive lower limit for a numeric node.       | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) | `stencila:minimum`                   | -                                        |
| `exclusiveMinimum` | The exclusive lower limit for a numeric node.       | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) | `stencila:exclusiveMinimum`          | `exclusive-minimum`, `exclusive_minimum` |
| `maximum`          | The inclusive upper limit for a numeric node.       | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) | `stencila:maximum`                   | -                                        |
| `exclusiveMaximum` | The exclusive upper limit for a numeric node.       | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) | `stencila:exclusiveMaximum`          | `exclusive-maximum`, `exclusive_maximum` |
| `multipleOf`       | A number that a numeric node must be a multiple of. | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) | `stencila:multipleOf`                | `multiple-of`, `multiple_of`             |

# Related

The `IntegerValidator` type is related to these types:

- Parents: [`NumberValidator`](./number-validator.md)
- Children: none

# Formats

The `IntegerValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                            | Notes |
| ------------------------------------------------ | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                    |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |                                    |
| [JATS](../formats/jats.md)                       |              |              |                                    |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              | Encoded using implemented function |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |                                    |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |                                    |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |                                    |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |                                    |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                    |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                    |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                    |
| [Citation File Format](../formats/cff.md)        |              |              |                                    |
| [CSV](../formats/csv.md)                         |              |              |                                    |
| [TSV](../formats/tsv.md)                         |              |              |                                    |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                    |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                    |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                    |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                    |
| [Directory](../formats/directory.md)             |              |              |                                    |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                    |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                    |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                    |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                    |
| [Email HTML](../formats/email.html.md)           |              |              |                                    |
| [MJML](../formats/mjml.md)                       |              |              |                                    |

# Bindings

The `IntegerValidator` type is represented in:

- [JSON-LD](https://stencila.org/IntegerValidator.jsonld)
- [JSON Schema](https://stencila.org/IntegerValidator.schema.json)
- Python class [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/integer_validator.py)
- Rust struct [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/integer_validator.rs)
- TypeScript class [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/IntegerValidator.ts)

# Source

This documentation was generated from [`IntegerValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/IntegerValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
