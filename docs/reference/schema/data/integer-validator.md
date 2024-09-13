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

| Format                                                                                               | Encoding     | Decoding  | Status              | Notes                              |
| ---------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ---------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |           | 🚧 Under development |                                    |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |           | 🚧 Under development |                                    |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |           | 🚧 Under development |                                    |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | ⚠️ High loss |           | ⚠️ Alpha            | Encoded using implemented function |
| [MyST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)                | ⚠️ High loss |           | ⚠️ Alpha            |                                    |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |           | ⚠️ Alpha            |                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |                                    |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |           | 🚧 Under development |                                    |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |           | 🚧 Under development |                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |           | 🟢 Stable            |                                    |

## Bindings

The `IntegerValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.org/IntegerValidator.jsonld)
- [JSON Schema](https://stencila.org/IntegerValidator.schema.json)
- Python class [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/integer_validator.py)
- Rust struct [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/integer_validator.rs)
- TypeScript class [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/IntegerValidator.ts)

## Source

This documentation was generated from [`IntegerValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/IntegerValidator.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
