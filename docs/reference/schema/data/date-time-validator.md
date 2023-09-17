---
title:
- type: Text
  value: DateTimeValidator
---

# Date Time Validator

**A validator specifying the constraints on a date-time.**

**`@id`**: `stencila:DateTimeValidator`

## Properties

The `DateTimeValidator` type has these properties:

| Name    | `@id`                                | Type                                                                    | Description                                | Inherited from                                                                             |
| ------- | ------------------------------------ | ----------------------------------------------------------------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------ |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)      | The identifier for this item               | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)                        |
| minimum | `stencila:minimum`                   | [`DateTime`](https://stencila.dev/docs/reference/schema/data/date-time) | The inclusive lower limit for a date-time. | [`DateTimeValidator`](https://stencila.dev/docs/reference/schema/data/date-time-validator) |
| maximum | `stencila:maximum`                   | [`DateTime`](https://stencila.dev/docs/reference/schema/data/date-time) | The inclusive upper limit for a date-time. | [`DateTimeValidator`](https://stencila.dev/docs/reference/schema/data/date-time-validator) |

## Related

The `DateTimeValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `DateTimeValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `DateTimeValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/DateTimeValidator.jsonld)
- [JSON Schema](https://stencila.dev/DateTimeValidator.schema.json)
- Python class [`DateTimeValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/date_time_validator.py)
- Rust struct [`DateTimeValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/date_time_validator.rs)
- TypeScript class [`DateTimeValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/DateTimeValidator.ts)

## Source

This documentation was generated from [`DateTimeValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/DateTimeValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).