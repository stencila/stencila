---
title:
- type: Text
  value: Timestamp
---

# Timestamp

**A value that represents a point in time**

**`@id`**: [`schema:Timestamp`](https://schema.org/Timestamp)

## Properties

The `Timestamp` type has these properties:

| Name     | `@id`                                      | Type                                                                    | Description                                                                      | Inherited from                                                           |
| -------- | ------------------------------------------ | ----------------------------------------------------------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| id       | [`schema:id`](https://schema.org/id)       | [`String`](https://stencila.dev/docs/reference/schema/data/string)      | The identifier for this item                                                     | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)      |
| value    | [`schema:value`](https://schema.org/value) | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer)    | The time, in `timeUnit`s, before or after the Unix Epoch (1970-01-01T00:00:00Z). | [`Timestamp`](https://stencila.dev/docs/reference/schema/data/timestamp) |
| timeUnit | `stencila:timeUnit`                        | [`TimeUnit`](https://stencila.dev/docs/reference/schema/data/time-unit) | The time unit that the `value` represents.                                       | [`Timestamp`](https://stencila.dev/docs/reference/schema/data/timestamp) |

## Related

The `Timestamp` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `Timestamp` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `Timestamp` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Timestamp.jsonld)
- [JSON Schema](https://stencila.dev/Timestamp.schema.json)
- Python class [`Timestamp`](https://github.com/stencila/stencila/blob/main/python/stencila/types/timestamp.py)
- Rust struct [`Timestamp`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/timestamp.rs)
- TypeScript class [`Timestamp`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Timestamp.ts)

## Source

This documentation was generated from [`Timestamp.yaml`](https://github.com/stencila/stencila/blob/main/schema/Timestamp.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).