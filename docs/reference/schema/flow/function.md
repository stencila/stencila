# Function

**A function with a name, which might take Parameters and return a value of a certain type.**

**`@id`**: `stencila:Function`

This type is marked as unstable and is subject to change.

## Properties

The `Function` type has these properties:

| Name         | Aliases     | `@id`                                    | Type                                                                                                   | Description                      | Inherited from                                                                                   |
| ------------ | ----------- | ---------------------------------------- | ------------------------------------------------------------------------------------------------------ | -------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`         | -           | [`schema:id`](https://schema.org/id)     | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)        | The identifier for this item.    | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `name`       | -           | [`schema:name`](https://schema.org/name) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)        | The name of the function.        | -                                                                                                |
| `parameters` | `parameter` | `stencila:parameters`                    | [`Parameter`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/parameter.md)* | The parameters of the function.  | -                                                                                                |
| `returns`    | -           | `stencila:returns`                       | [`Validator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/validator.md)  | The return type of the function. | -                                                                                                |

## Related

The `Function` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Function` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding     | Decoding  | Status              | Notes |
| -------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.md)           | 🟢 No loss    |           | 🚧 Under development |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss   |           | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |              |           | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss   |           | 🟢 Stable            |       |

## Bindings

The `Function` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Function.jsonld)
- [JSON Schema](https://stencila.org/Function.schema.json)
- Python class [`Function`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/function.py)
- Rust struct [`Function`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/function.rs)
- TypeScript class [`Function`](https://github.com/stencila/stencila/blob/main/ts/src/types/Function.ts)

## Source

This documentation was generated from [`Function.yaml`](https://github.com/stencila/stencila/blob/main/schema/Function.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).