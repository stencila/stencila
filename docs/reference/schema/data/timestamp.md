# Timestamp

**A value that represents a point in time**

**`@id`**: [`schema:Timestamp`](https://schema.org/Timestamp)

## Properties

The `Timestamp` type has these properties:

| Name     | `@id`                                      | Type                                                                                                 | Description                                                                      | Inherited from                                                                                        |
| -------- | ------------------------------------------ | ---------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| id       | [`schema:id`](https://schema.org/id)       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)      | The identifier for this item                                                     | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)      |
| value    | [`schema:value`](https://schema.org/value) | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)    | The time, in `timeUnit`s, before or after the Unix Epoch (1970-01-01T00:00:00Z). | [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md) |
| timeUnit | `stencila:timeUnit`                        | [`TimeUnit`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time-unit.md) | The time unit that the `value` represents.                                       | [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md) |

## Related

The `Timestamp` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Timestamp` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                            | Encoding       | Decoding     | Status                 | Notes |
| ------------------------------------------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/HTML.md)             | 🔷 Low loss     |              | 🚧 Under development    |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JATS.md)             | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Markdown.md)     | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Plain text.md) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JSON.md)             | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JSON5.md)           | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/YAML.md)             | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Debug.md)           | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `Timestamp` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Timestamp.jsonld)
- [JSON Schema](https://stencila.dev/Timestamp.schema.json)
- Python class [`Timestamp`](https://github.com/stencila/stencila/blob/main/python/stencila/types/timestamp.py)
- Rust struct [`Timestamp`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/timestamp.rs)
- TypeScript class [`Timestamp`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Timestamp.ts)

## Source

This documentation was generated from [`Timestamp.yaml`](https://github.com/stencila/stencila/blob/main/schema/Timestamp.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).