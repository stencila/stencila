# Array Hint

**A hint to the content of an `Array`.**

**`@id`**: `stencila:ArrayHint`

## Properties

The `ArrayHint` type has these properties:

| Name        | Aliases                                                          | `@id`                                | Type                                                                                                  | Description                                | Inherited from                                                                                   |
| ----------- | ---------------------------------------------------------------- | ------------------------------------ | ----------------------------------------------------------------------------------------------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| `id`        | -                                                                | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)       | The identifier for this item.              | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `length`    | -                                                                | `stencila:length`                    | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)     | The length (number of items) of the array. | -                                                                                                |
| `itemTypes` | `item-types`, `item_types`, `itemType`, `item-type`, `item_type` | `stencila:itemTypes`                 | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)*      | The distinct types of the array items.     | -                                                                                                |
| `minimum`   | -                                                                | `stencila:minimum`                   | [`Primitive`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/primitive.md) | The minimum value in the array.            | -                                                                                                |
| `maximum`   | -                                                                | `stencila:maximum`                   | [`Primitive`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/primitive.md) | The maximum value in the array.            | -                                                                                                |
| `nulls`     | -                                                                | `stencila:nulls`                     | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)     | The number of `Null` values in the array.  | -                                                                                                |

## Related

The `ArrayHint` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `ArrayHint` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding  | Status              | Notes |
| ---------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |           | 🚧 Under development |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |           | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |           | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [MyST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)                | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |           | 🚧 Under development |       |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |           | 🚧 Under development |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |           | 🟢 Stable            |       |

## Bindings

The `ArrayHint` type is represented in these bindings:

- [JSON-LD](https://stencila.org/ArrayHint.jsonld)
- [JSON Schema](https://stencila.org/ArrayHint.schema.json)
- Python class [`ArrayHint`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/array_hint.py)
- Rust struct [`ArrayHint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/array_hint.rs)
- TypeScript class [`ArrayHint`](https://github.com/stencila/stencila/blob/main/ts/src/types/ArrayHint.ts)

## Source

This documentation was generated from [`ArrayHint.yaml`](https://github.com/stencila/stencila/blob/main/schema/ArrayHint.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
