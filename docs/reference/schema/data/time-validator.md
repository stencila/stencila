# Time Validator

**A validator specifying the constraints on a time.**

**`@id`**: `stencila:TimeValidator`

## Properties

The `TimeValidator` type has these properties:

| Name      | Aliases | `@id`                                | Type                                                                                            | Description                           | Inherited from                                                                                   |
| --------- | ------- | ------------------------------------ | ----------------------------------------------------------------------------------------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`      | -       | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item.         | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `minimum` | -       | `stencila:minimum`                   | [`Time`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time.md)     | The inclusive lower limit for a time. | -                                                                                                |
| `maximum` | -       | `stencila:maximum`                   | [`Time`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time.md)     | The inclusive upper limit for a time. | -                                                                                                |

## Related

The `TimeValidator` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `TimeValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `TimeValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.org/TimeValidator.jsonld)
- [JSON Schema](https://stencila.org/TimeValidator.schema.json)
- Python class [`TimeValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/time_validator.py)
- Rust struct [`TimeValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/time_validator.rs)
- TypeScript class [`TimeValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/TimeValidator.ts)

## Source

This documentation was generated from [`TimeValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/TimeValidator.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
