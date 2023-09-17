---
title:
- type: Text
  value: Function
---

# Function

**A function with a name, which might take Parameters and return a value of a certain type.**

**`@id`**: `stencila:Function`

This type is marked as experimental and is likely to change.

## Properties

The `Function` type has these properties:

| Name       | `@id`                                    | Type                                                                      | Description                      | Inherited from                                                         |
| ---------- | ---------------------------------------- | ------------------------------------------------------------------------- | -------------------------------- | ---------------------------------------------------------------------- |
| id         | [`schema:id`](https://schema.org/id)     | [`String`](https://stencila.dev/docs/reference/schema/data/string)        | The identifier for this item     | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)    |
| name       | [`schema:name`](https://schema.org/name) | [`String`](https://stencila.dev/docs/reference/schema/data/string)        | The name of the function.        | [`Function`](https://stencila.dev/docs/reference/schema/flow/function) |
| parameters | `stencila:parameters`                    | [`Parameter`](https://stencila.dev/docs/reference/schema/flow/parameter)* | The parameters of the function.  | [`Function`](https://stencila.dev/docs/reference/schema/flow/function) |
| returns    | `stencila:returns`                       | [`Validator`](https://stencila.dev/docs/reference/schema/data/validator)  | The return type of the function. | [`Function`](https://stencila.dev/docs/reference/schema/flow/function) |

## Related

The `Function` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `Function` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `Function` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Function.jsonld)
- [JSON Schema](https://stencila.dev/Function.schema.json)
- Python class [`Function`](https://github.com/stencila/stencila/blob/main/python/stencila/types/function.py)
- Rust struct [`Function`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/function.rs)
- TypeScript class [`Function`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Function.ts)

## Source

This documentation was generated from [`Function.yaml`](https://github.com/stencila/stencila/blob/main/schema/Function.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).