---
title:
- type: Text
  value: Date
---

# Date

**A calendar date encoded as a ISO 8601 string.**

**`@id`**: [`schema:Date`](https://schema.org/Date)

## Properties

The `Date` type has these properties:

| Name  | `@id`                                      | Type                                                               | Description                     | Inherited from                                                      |
| ----- | ------------------------------------------ | ------------------------------------------------------------------ | ------------------------------- | ------------------------------------------------------------------- |
| id    | [`schema:id`](https://schema.org/id)       | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item    | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| value | [`schema:value`](https://schema.org/value) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The date as an ISO 8601 string. | [`Date`](https://stencila.dev/docs/reference/schema/data/date)      |

## Related

The `Date` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `Date` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `Date` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Date.jsonld)
- [JSON Schema](https://stencila.dev/Date.schema.json)
- Python class [`Date`](https://github.com/stencila/stencila/blob/main/python/stencila/types/date.py)
- Rust struct [`Date`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/date.rs)
- TypeScript class [`Date`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Date.ts)

## Source

This documentation was generated from [`Date.yaml`](https://github.com/stencila/stencila/blob/main/schema/Date.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).