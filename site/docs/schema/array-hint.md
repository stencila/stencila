---
title: Array Hint
description: A hint to the content of an `Array`.
---

# Properties

The `ArrayHint` type has these properties:

| Name        | Description                                | Type                          | Inherited from          |
| ----------- | ------------------------------------------ | ----------------------------- | ----------------------- |
| `id`        | The identifier for this item.              | [`String`](./string.md)       | [`Entity`](./entity.md) |
| `length`    | The length (number of items) of the array. | [`Integer`](./integer.md)     | -                       |
| `itemTypes` | The distinct types of the array items.     | [`String`](./string.md)*      | -                       |
| `minimum`   | The minimum value in the array.            | [`Primitive`](./primitive.md) | -                       |
| `maximum`   | The maximum value in the array.            | [`Primitive`](./primitive.md) | -                       |
| `nulls`     | The number of `Null` values in the array.  | [`Integer`](./integer.md)     | -                       |

# Related

The `ArrayHint` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ArrayHint` type is represented in:

- [JSON-LD](https://stencila.org/ArrayHint.jsonld)
- [JSON Schema](https://stencila.org/ArrayHint.schema.json)
- Python class [`ArrayHint`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/array_hint.py)
- Rust struct [`ArrayHint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/array_hint.rs)
- TypeScript class [`ArrayHint`](https://github.com/stencila/stencila/blob/main/ts/src/types/ArrayHint.ts)

# Source

This documentation was generated from [`ArrayHint.yaml`](https://github.com/stencila/stencila/blob/main/schema/ArrayHint.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
