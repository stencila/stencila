# Integer Validator

**A validator specifying the constraints on an integer node.**

A node will be valid if it is a number with no fractional part and meets any additional constraints,
such as `multipleOf`, specified in the validator.
Analogous to the JSON Schema `integer` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).


**`@id`**: `stencila:IntegerValidator`

## Properties

The `IntegerValidator` type has these properties:

| Name               | Aliases                                  | `@id`                                | Type                                                                                            | Description                                         | Inherited from                                                                                                     |
| ------------------ | ---------------------------------------- | ------------------------------------ | ----------------------------------------------------------------------------------------------- | --------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`               | -                                        | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item.                       | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)                   |
| `minimum`          | -                                        | `stencila:minimum`                   | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | The inclusive lower limit for a numeric node.       | [`NumberValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number-validator.md) |
| `exclusiveMinimum` | `exclusive-minimum`, `exclusive_minimum` | `stencila:exclusiveMinimum`          | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | The exclusive lower limit for a numeric node.       | [`NumberValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number-validator.md) |
| `maximum`          | -                                        | `stencila:maximum`                   | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | The inclusive upper limit for a numeric node.       | [`NumberValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number-validator.md) |
| `exclusiveMaximum` | `exclusive-maximum`, `exclusive_maximum` | `stencila:exclusiveMaximum`          | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | The exclusive upper limit for a numeric node.       | [`NumberValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number-validator.md) |
| `multipleOf`       | `multiple-of`, `multiple_of`             | `stencila:multipleOf`                | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | A number that a numeric node must be a multiple of. | [`NumberValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number-validator.md) |

## Related

The `IntegerValidator` type is related to these types:

- Parents: [`NumberValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number-validator.md)
- Children: none

## Formats

The `IntegerValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding   | Status              | Notes                              |
| ---------------------------------------------------------------------------------------------------- | ------------ | ---------- | ------------------- | ---------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |            | 🔶 Beta              |                                    |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |            | 🚧 Under development |                                    |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |            | 🚧 Under development |                                    |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/md.md)              | ⚠️ High loss |            | 🔶 Beta              | Encoded using implemented function |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [Quarto Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/qmd.md)      | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [LaTeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/latex.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [PDF](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pdf.md)                  | 🔷 Low loss   |            | 🚧 Under development |                                    |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |            | 🔶 Beta              |                                    |
| [IPYNB](https://github.com/stencila/stencila/blob/main/docs/reference/formats/ipynb.md)              | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [Microsoft Word DOCX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/docx.md) | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [OpenDocument ODT](https://github.com/stencila/stencila/blob/main/docs/reference/formats/odt.md)     | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [TeX](https://github.com/stencila/stencila/blob/main/docs/reference/formats/tex.md)                  | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss  | 🔶 Beta              |                                    |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss  | 🟢 Stable            |                                    |
| [Lexical JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/lexical.md)     | 🔷 Low loss   | 🔷 Low loss | ⚠️ Alpha            |                                    |
| [Koenig JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/koenig.md)       | 🔷 Low loss   | 🔷 Low loss | ⚠️ Alpha            |                                    |
| [Pandoc AST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/pandoc.md)        | 🔷 Low loss   | 🔷 Low loss | 🚧 Under development |                                    |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |            | 🚧 Under development |                                    |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |            | ⚠️ Alpha            |                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |            | 🟢 Stable            |                                    |

## Bindings

The `IntegerValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.org/IntegerValidator.jsonld)
- [JSON Schema](https://stencila.org/IntegerValidator.schema.json)
- Python class [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/integer_validator.py)
- Rust struct [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/integer_validator.rs)
- TypeScript class [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/IntegerValidator.ts)

## Source

This documentation was generated from [`IntegerValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/IntegerValidator.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
