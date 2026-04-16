---
title: Array Hint
description: A concise summary of the values and structure of an `Array`.
---

This is a type used in Stencila Schema for providing a concise summary of the values
and structure of an [`Array`](./array.md).

It exists to support both human and machine understanding of array data,
including schema inference, user-interface hints, and code generation
workflows. Rather than constraining data directly, it summarizes observed or
inferred characteristics that can guide inspection, editing, and downstream
uses such as selecting visualization strategies for large datasets.

Key properties describe item-level hints and observed array characteristics.


# Properties

The `ArrayHint` type has these properties:

| Name        | Description                                | Type                          | Inherited from          |
| ----------- | ------------------------------------------ | ----------------------------- | ----------------------- |
| `length`    | The length (number of items) of the array. | [`Integer`](./integer.md)     | -                       |
| `itemTypes` | The distinct types of the array items.     | [`String`](./string.md)*      | -                       |
| `minimum`   | The minimum value in the array.            | [`Primitive`](./primitive.md) | -                       |
| `maximum`   | The maximum value in the array.            | [`Primitive`](./primitive.md) | -                       |
| `nulls`     | The number of `Null` values in the array.  | [`Integer`](./integer.md)     | -                       |
| `id`        | The identifier for this item.              | [`String`](./string.md)       | [`Entity`](./entity.md) |

# Related

The `ArrayHint` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ArrayHint` type is represented in:

- [JSON-LD](https://stencila.org/ArrayHint.jsonld)
- [JSON Schema](https://stencila.org/ArrayHint.schema.json)
- Python class [`ArrayHint`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ArrayHint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/array_hint.rs)
- TypeScript class [`ArrayHint`](https://github.com/stencila/stencila/blob/main/ts/src/types/ArrayHint.ts)

***

This documentation was generated from [`ArrayHint.yaml`](https://github.com/stencila/stencila/blob/main/schema/ArrayHint.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
