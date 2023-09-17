---
title:
- type: Text
  value: DateTime
---

# Date Time

**A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.**

**`@id`**: [`schema:DateTime`](https://schema.org/DateTime)

## Properties

The `DateTime` type has these properties:

| Name  | `@id`                                      | Type                                                               | Description                     | Inherited from                                                          |
| ----- | ------------------------------------------ | ------------------------------------------------------------------ | ------------------------------- | ----------------------------------------------------------------------- |
| id    | [`schema:id`](https://schema.org/id)       | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item    | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)     |
| value | [`schema:value`](https://schema.org/value) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The date as an ISO 8601 string. | [`DateTime`](https://stencila.dev/docs/reference/schema/data/date-time) |

## Related

The `DateTime` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `DateTime` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `DateTime` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/DateTime.jsonld)
- [JSON Schema](https://stencila.dev/DateTime.schema.json)
- Python class [`DateTime`](https://github.com/stencila/stencila/blob/main/python/stencila/types/date_time.py)
- Rust struct [`DateTime`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/date_time.rs)
- TypeScript class [`DateTime`](https://github.com/stencila/stencila/blob/main/typescript/src/types/DateTime.ts)

## Source

This documentation was generated from [`DateTime.yaml`](https://github.com/stencila/stencila/blob/main/schema/DateTime.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).