---
title:
- type: Text
  value: Duration
---

# Duration

**A value that represents the difference between two timestamps**

**`@id`**: [`schema:Duration`](https://schema.org/Duration)

## Properties

The `Duration` type has these properties:

| Name     | `@id`                                      | Type                                                                    | Description                                | Inherited from                                                         |
| -------- | ------------------------------------------ | ----------------------------------------------------------------------- | ------------------------------------------ | ---------------------------------------------------------------------- |
| id       | [`schema:id`](https://schema.org/id)       | [`String`](https://stencila.dev/docs/reference/schema/data/string)      | The identifier for this item               | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)    |
| value    | [`schema:value`](https://schema.org/value) | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer)    | The time difference in `timeUnit`s.        | [`Duration`](https://stencila.dev/docs/reference/schema/data/duration) |
| timeUnit | `stencila:timeUnit`                        | [`TimeUnit`](https://stencila.dev/docs/reference/schema/data/time-unit) | The time unit that the `value` represents. | [`Duration`](https://stencila.dev/docs/reference/schema/data/duration) |

## Related

The `Duration` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `Duration` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `Duration` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Duration.jsonld)
- [JSON Schema](https://stencila.dev/Duration.schema.json)
- Python class [`Duration`](https://github.com/stencila/stencila/blob/main/python/stencila/types/duration.py)
- Rust struct [`Duration`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/duration.rs)
- TypeScript class [`Duration`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Duration.ts)

## Source

This documentation was generated from [`Duration.yaml`](https://github.com/stencila/stencila/blob/main/schema/Duration.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).