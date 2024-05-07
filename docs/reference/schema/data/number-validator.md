# Number Validator

**A validator specifying the constraints on a numeric node.**

A node will be valid if it is a number that meets the `maximum`, `multipleOf` etc properties.
Analogous to the JSON Schema `number` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).
Note that the `IntegerValidator` type extends this validator with the additional
constraint that the number have no fractional part.


**`@id`**: `stencila:NumberValidator`

## Properties

The `NumberValidator` type has these properties:

| Name               | Aliases                                  | `@id`                                | Type                                                                                            | Description                                         | Inherited from                                                                                   |
| ------------------ | ---------------------------------------- | ------------------------------------ | ----------------------------------------------------------------------------------------------- | --------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`               | -                                        | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item.                       | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `minimum`          | -                                        | `stencila:minimum`                   | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | The inclusive lower limit for a numeric node.       | -                                                                                                |
| `exclusiveMinimum` | `exclusive-minimum`, `exclusive_minimum` | `stencila:exclusiveMinimum`          | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | The exclusive lower limit for a numeric node.       | -                                                                                                |
| `maximum`          | -                                        | `stencila:maximum`                   | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | The inclusive upper limit for a numeric node.       | -                                                                                                |
| `exclusiveMaximum` | `exclusive-maximum`, `exclusive_maximum` | `stencila:exclusiveMaximum`          | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | The exclusive upper limit for a numeric node.       | -                                                                                                |
| `multipleOf`       | `multiple-of`, `multiple_of`             | `stencila:multipleOf`                | [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md) | A number that a numeric node must be a multiple of. | -                                                                                                |

## Related

The `NumberValidator` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer-validator.md)

## Formats

The `NumberValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding  | Status              | Notes                              |
| ---------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ---------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |           | 🚧 Under development |                                    |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |           | 🚧 Under development |                                    |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |           | 🚧 Under development |                                    |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | ⚠️ High loss |           | ⚠️ Alpha            | Encoded using implemented function |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |           | ⚠️ Alpha            |                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |                                    |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |           | 🚧 Under development |                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |           | 🟢 Stable            |                                    |

## Bindings

The `NumberValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.org/NumberValidator.jsonld)
- [JSON Schema](https://stencila.org/NumberValidator.schema.json)
- Python class [`NumberValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/number_validator.py)
- Rust struct [`NumberValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/number_validator.rs)
- TypeScript class [`NumberValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/NumberValidator.ts)

## Source

This documentation was generated from [`NumberValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/NumberValidator.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
