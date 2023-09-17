---
title:
- type: Text
  value: DateValidator
---

# Date Validator

**A validator specifying the constraints on a date.**

**`@id`**: `stencila:DateValidator`

## Properties

The `DateValidator` type has these properties:

| Name    | `@id`                                | Type                                                               | Description                           | Inherited from                                                                    |
| ------- | ------------------------------------ | ------------------------------------------------------------------ | ------------------------------------- | --------------------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item          | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)               |
| minimum | `stencila:minimum`                   | [`Date`](https://stencila.dev/docs/reference/schema/data/date)     | The inclusive lower limit for a date. | [`DateValidator`](https://stencila.dev/docs/reference/schema/data/date-validator) |
| maximum | `stencila:maximum`                   | [`Date`](https://stencila.dev/docs/reference/schema/data/date)     | The inclusive upper limit for a date. | [`DateValidator`](https://stencila.dev/docs/reference/schema/data/date-validator) |

## Related

The `DateValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `DateValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `DateValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/DateValidator.jsonld)
- [JSON Schema](https://stencila.dev/DateValidator.schema.json)
- Python class [`DateValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/date_validator.py)
- Rust struct [`DateValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/date_validator.rs)
- TypeScript class [`DateValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/DateValidator.ts)

## Source

This documentation was generated from [`DateValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/DateValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).