---
title:
- type: Text
  value: Time
---

# Time

**A point in time recurring on multiple days**

**`@id`**: [`schema:Time`](https://schema.org/Time)

## Properties

The `Time` type has these properties:

| Name  | `@id`                                      | Type                                                               | Description                                                     | Inherited from                                                      |
| ----- | ------------------------------------------ | ------------------------------------------------------------------ | --------------------------------------------------------------- | ------------------------------------------------------------------- |
| id    | [`schema:id`](https://schema.org/id)       | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item                                    | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| value | [`schema:value`](https://schema.org/value) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The time of day as a string in format `hh:mm:ss[Z\|(+\|-)hh:mm]`. | [`Time`](https://stencila.dev/docs/reference/schema/data/time)      |

## Related

The `Time` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `Time` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `Time` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Time.jsonld)
- [JSON Schema](https://stencila.dev/Time.schema.json)
- Python class [`Time`](https://github.com/stencila/stencila/blob/main/python/stencila/types/time.py)
- Rust struct [`Time`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/time.rs)
- TypeScript class [`Time`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Time.ts)

## Source

This documentation was generated from [`Time.yaml`](https://github.com/stencila/stencila/blob/main/schema/Time.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).