---
title:
- type: Text
  value: TimeValidator
---

# Time Validator

**A validator specifying the constraints on a time.**

**`@id`**: `stencila:TimeValidator`

## Properties

The `TimeValidator` type has these properties:

| Name    | `@id`                                | Type                                                               | Description                           | Inherited from                                                                    |
| ------- | ------------------------------------ | ------------------------------------------------------------------ | ------------------------------------- | --------------------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item          | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)               |
| minimum | `stencila:minimum`                   | [`Time`](https://stencila.dev/docs/reference/schema/data/time)     | The inclusive lower limit for a time. | [`TimeValidator`](https://stencila.dev/docs/reference/schema/data/time-validator) |
| maximum | `stencila:maximum`                   | [`Time`](https://stencila.dev/docs/reference/schema/data/time)     | The inclusive upper limit for a time. | [`TimeValidator`](https://stencila.dev/docs/reference/schema/data/time-validator) |

## Related

The `TimeValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `TimeValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `TimeValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/TimeValidator.jsonld)
- [JSON Schema](https://stencila.dev/TimeValidator.schema.json)
- Python class [`TimeValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/time_validator.py)
- Rust struct [`TimeValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/time_validator.rs)
- TypeScript class [`TimeValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/TimeValidator.ts)

## Source

This documentation was generated from [`TimeValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/TimeValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).