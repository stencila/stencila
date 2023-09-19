# Timestamp Validator

**A validator specifying the constraints on a timestamp.**

**`@id`**: `stencila:TimestampValidator`

## Properties

The `TimestampValidator` type has these properties:

| Name      | `@id`                                | Type                                                                                                  | Description                                 | Inherited from                                                                                                           |
| --------- | ------------------------------------ | ----------------------------------------------------------------------------------------------------- | ------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| id        | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)       | The identifier for this item                | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)                         |
| timeUnits | `stencila:timeUnits`                 | [`TimeUnit`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time-unit.md)* | The time units that the timestamp can have. | [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp-validator.md) |
| minimum   | `stencila:minimum`                   | [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md) | The inclusive lower limit for a timestamp.  | [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp-validator.md) |
| maximum   | `stencila:maximum`                   | [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md) | The inclusive upper limit for a timestamp.  | [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp-validator.md) |

## Related

The `TimestampValidator` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `TimestampValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding     | Status                 | Notes |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss       |              | 🚧 Under development    |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         |                  |              | 🚧 Under development    |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | ⚠️ High loss     |              | 🚧 Under development    |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |              | ⚠️ Alpha               |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |              | 🟢 Stable               |       |

## Bindings

The `TimestampValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/TimestampValidator.jsonld)
- [JSON Schema](https://stencila.dev/TimestampValidator.schema.json)
- Python class [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/timestamp_validator.py)
- Rust struct [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/timestamp_validator.rs)
- TypeScript class [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/TimestampValidator.ts)

## Source

This documentation was generated from [`TimestampValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/TimestampValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).