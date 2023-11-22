# Tuple Validator

**A validator specifying constraints on an array of heterogeneous items.**

**`@id`**: `stencila:TupleValidator`

## Properties

The `TupleValidator` type has these properties:

| Name    | Aliases | `@id`                                                          | Type                                                                                                   | Description                                                                             | Inherited from                                                                                   |
| ------- | ------- | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`    | -       | [`schema:id`](https://schema.org/id)                           | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)        | The identifier for this item.                                                           | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `items` | `item`  | [`schema:itemListElement`](https://schema.org/itemListElement) | [`Validator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/validator.md)* | An array of validators specifying the constraints on each successive item in the array. | -                                                                                                |

## Related

The `TupleValidator` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `TupleValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding         | Decoding     | Status                 | Notes |
| -------------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss       |              | 🚧 Under development    |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |                  |              | 🚧 Under development    |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss     |              | ⚠️ Alpha               |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss     |              | ⚠️ Alpha               |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss       |              | 🟢 Stable               |       |

## Bindings

The `TupleValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/TupleValidator.jsonld)
- [JSON Schema](https://stencila.dev/TupleValidator.schema.json)
- Python class [`TupleValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/tuple_validator.py)
- Rust struct [`TupleValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/tuple_validator.rs)
- TypeScript class [`TupleValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/TupleValidator.ts)

## Source

This documentation was generated from [`TupleValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/TupleValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).