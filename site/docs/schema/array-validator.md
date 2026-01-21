---
title: Array Validator
description: A validator specifying constraints on an array node.
---

# Properties

The `ArrayValidator` type has these properties:

| Name             | Description                                                                                 | Type                          | Inherited from          |
| ---------------- | ------------------------------------------------------------------------------------------- | ----------------------------- | ----------------------- |
| `id`             | The identifier for this item.                                                               | [`String`](./string.md)       | [`Entity`](./entity.md) |
| `itemsNullable`  | Whether items can have the value `Node::Null`                                               | [`Boolean`](./boolean.md)     | -                       |
| `itemsValidator` | Another validator node specifying the constraints on all items in the array.                | [`Validator`](./validator.md) | -                       |
| `contains`       | An array node is valid if at least one of its items is valid against the `contains` schema. | [`Validator`](./validator.md) | -                       |
| `minItems`       | An array node is valid if its size is greater than, or equal to, this value.                | [`Integer`](./integer.md)     | -                       |
| `maxItems`       | An array node is valid if its size is less than, or equal to, this value.                   | [`Integer`](./integer.md)     | -                       |
| `uniqueItems`    | A flag to indicate that each value in the array should be unique.                           | [`Boolean`](./boolean.md)     | -                       |

# Related

The `ArrayValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ArrayValidator` type is represented in:

- [JSON-LD](https://stencila.org/ArrayValidator.jsonld)
- [JSON Schema](https://stencila.org/ArrayValidator.schema.json)
- Python class [`ArrayValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/array_validator.py)
- Rust struct [`ArrayValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/array_validator.rs)
- TypeScript class [`ArrayValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/ArrayValidator.ts)

# Source

This documentation was generated from [`ArrayValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/ArrayValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
