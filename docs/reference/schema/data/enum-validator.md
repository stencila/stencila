# Enum Validator

**A schema specifying that a node must be one of several values.**

Analogous to the JSON Schema [`enum` keyword](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.2).

**`@id`**: `stencila:EnumValidator`

## Properties

The `EnumValidator` type has these properties:

| Name     | Aliases | `@id`                                | Type                                                                                            | Description                                            | Inherited from                                                                                   |
| -------- | ------- | ------------------------------------ | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| `id`     | -       | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item.                          | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `values` | `value` | `stencila:values`                    | [`Node`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/node.md)*   | A node is valid if it is equal to any of these values. | -                                                                                                |

## Related

The `EnumValidator` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `EnumValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `EnumValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.org/EnumValidator.jsonld)
- [JSON Schema](https://stencila.org/EnumValidator.schema.json)
- Python class [`EnumValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/enum_validator.py)
- Rust struct [`EnumValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/enum_validator.rs)
- TypeScript class [`EnumValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/EnumValidator.ts)

## Source

This documentation was generated from [`EnumValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/EnumValidator.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).