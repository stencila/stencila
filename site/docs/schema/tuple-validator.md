---
title: Tuple Validator
description: A validator specifying constraints on an array of heterogeneous items.
---

# Properties

The `TupleValidator` type has these properties:

| Name    | Description                                                                             | Type                           | Inherited from          |
| ------- | --------------------------------------------------------------------------------------- | ------------------------------ | ----------------------- |
| `id`    | The identifier for this item.                                                           | [`String`](./string.md)        | [`Entity`](./entity.md) |
| `items` | An array of validators specifying the constraints on each successive item in the array. | [`Validator`](./validator.md)* | -                       |

# Related

The `TupleValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `TupleValidator` type is represented in:

- [JSON-LD](https://stencila.org/TupleValidator.jsonld)
- [JSON Schema](https://stencila.org/TupleValidator.schema.json)
- Python class [`TupleValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/tuple_validator.py)
- Rust struct [`TupleValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/tuple_validator.rs)
- TypeScript class [`TupleValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/TupleValidator.ts)

# Source

This documentation was generated from [`TupleValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/TupleValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
