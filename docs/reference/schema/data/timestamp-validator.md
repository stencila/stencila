# Timestamp Validator

**A validator specifying the constraints on a timestamp.**

**`@id`**: `stencila:TimestampValidator`

## Properties

The `TimestampValidator` type has these properties:

| Name        | Aliases                                                          | `@id`                                | Type                                                                                                  | Description                                 | Inherited from                                                                                   |
| ----------- | ---------------------------------------------------------------- | ------------------------------------ | ----------------------------------------------------------------------------------------------------- | ------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`        | -                                                                | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)       | The identifier for this item.               | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `timeUnits` | `time-units`, `time_units`, `timeUnit`, `time-unit`, `time_unit` | `stencila:timeUnits`                 | [`TimeUnit`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time-unit.md)* | The time units that the timestamp can have. | -                                                                                                |
| `minimum`   | -                                                                | `stencila:minimum`                   | [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md) | The inclusive lower limit for a timestamp.  | -                                                                                                |
| `maximum`   | -                                                                | `stencila:maximum`                   | [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md) | The inclusive upper limit for a timestamp.  | -                                                                                                |

## Related

The `TimestampValidator` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `TimestampValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding     | Decoding  | Status              | Notes                              |
| -------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ---------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.md)           | 🟢 No loss    |           | 🚧 Under development |                                    |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss   |           | 🚧 Under development |                                    |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |              |           | 🚧 Under development |                                    |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss |           | ⚠️ Alpha            | Encoded using implemented function |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss |           | ⚠️ Alpha            |                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |                                    |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)    |              |           | 🚧 Under development |                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss   |           | 🟢 Stable            |                                    |

## Bindings

The `TimestampValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.org/TimestampValidator.jsonld)
- [JSON Schema](https://stencila.org/TimestampValidator.schema.json)
- Python class [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/timestamp_validator.py)
- Rust struct [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/timestamp_validator.rs)
- TypeScript class [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/TimestampValidator.ts)

## Source

This documentation was generated from [`TimestampValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/TimestampValidator.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).