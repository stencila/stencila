---
title:
- type: Text
  value: DurationValidator
---

# Duration Validator

**A validator specifying the constraints on a duration.**

**`@id`**: `stencila:DurationValidator`

## Properties

The `DurationValidator` type has these properties:

| Name      | `@id`                                | Type                                                                     | Description                                | Inherited from                                                                            |
| --------- | ------------------------------------ | ------------------------------------------------------------------------ | ------------------------------------------ | ----------------------------------------------------------------------------------------- |
| id        | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)       | The identifier for this item               | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)                       |
| timeUnits | `stencila:timeUnits`                 | [`TimeUnit`](https://stencila.dev/docs/reference/schema/data/time-unit)* | The time units that the duration can have. | [`DurationValidator`](https://stencila.dev/docs/reference/schema/data/duration-validator) |
| minimum   | `stencila:minimum`                   | [`Duration`](https://stencila.dev/docs/reference/schema/data/duration)   | The inclusive lower limit for a duration.  | [`DurationValidator`](https://stencila.dev/docs/reference/schema/data/duration-validator) |
| maximum   | `stencila:maximum`                   | [`Duration`](https://stencila.dev/docs/reference/schema/data/duration)   | The inclusive upper limit for a duration.  | [`DurationValidator`](https://stencila.dev/docs/reference/schema/data/duration-validator) |

## Related

The `DurationValidator` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `DurationValidator` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `DurationValidator` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/DurationValidator.jsonld)
- [JSON Schema](https://stencila.dev/DurationValidator.schema.json)
- Python class [`DurationValidator`](https://github.com/stencila/stencila/blob/main/python/stencila/types/duration_validator.py)
- Rust struct [`DurationValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/duration_validator.rs)
- TypeScript class [`DurationValidator`](https://github.com/stencila/stencila/blob/main/typescript/src/types/DurationValidator.ts)

## Source

This documentation was generated from [`DurationValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/DurationValidator.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).