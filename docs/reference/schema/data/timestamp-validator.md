---
title:
- type: Text
  value: TimestampValidator
---

# Timestamp Validator

**A validator specifying the constraints on a timestamp.**

**`@id`**: `stencila:TimestampValidator`

## Properties

The `TimestampValidator` type has these properties:

| Name      | `@id`                                | Type                                                                     | Description                                 | Inherited from                                                                              |
| --------- | ------------------------------------ | ------------------------------------------------------------------------ | ------------------------------------------- | ------------------------------------------------------------------------------------------- |
| id        | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)       | The identifier for this item                | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)                         |
| timeUnits | `stencila:timeUnits`                 | [`TimeUnit`](https://stencila.dev/docs/reference/schema/data/time-unit)* | The time units that the timestamp can have. | [`TimestampValidator`](https://stencila.dev/docs/reference/schema/data/timestamp-validator) |
| minimum   | `stencila:minimum`                   | [`Timestamp`](https://stencila.dev/docs/reference/schema/data/timestamp) | The inclusive lower limit for a timestamp.  | [`TimestampValidator`](https://stencila.dev/docs/reference/schema/data/timestamp-validator) |
| maximum   | `stencila:maximum`                   | [`Timestamp`](https://stencila.dev/docs/reference/schema/data/timestamp) | The inclusive upper limit for a timestamp.  | [`TimestampValidator`](https://stencila.dev/docs/reference/schema/data/timestamp-validator) |

## Related

The `TimestampValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `TimestampValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `TimestampValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/TimestampValidator.jsonld)
- [JSON Schema](https://stencila.dev/TimestampValidator.schema.json)
- Python class [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/timestamp_validator.py)
- Rust struct [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/timestamp_validator.rs)
- TypeScript class [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/TimestampValidator.ts)

## Source

This documentation was generated from [`TimestampValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/TimestampValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).