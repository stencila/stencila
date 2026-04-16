---
title: Tuple Validator
description: A validator specifying constraints on an array of heterogeneous items.
---

This is a type used in Stencila Schema for validating heterogeneous arrays.

It adapts tuple-style validation ideas from JSON Schema to Stencila's node
model, allowing arrays to be constrained positionally when each item has a
different expected type or validator. This is useful for structured arguments
and compact data records.

Key properties describe per-position validators and array-length constraints.


# Properties

The `TupleValidator` type has these properties:

| Name    | Description                                                                             | Type                           | Inherited from          |
| ------- | --------------------------------------------------------------------------------------- | ------------------------------ | ----------------------- |
| `items` | An array of validators specifying the constraints on each successive item in the array. | [`Validator`](./validator.md)* | -                       |
| `id`    | The identifier for this item.                                                           | [`String`](./string.md)        | [`Entity`](./entity.md) |

# Related

The `TupleValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `TupleValidator` type is represented in:

- [JSON-LD](https://stencila.org/TupleValidator.jsonld)
- [JSON Schema](https://stencila.org/TupleValidator.schema.json)
- Python class [`TupleValidator`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`TupleValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/tuple_validator.rs)
- TypeScript class [`TupleValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/TupleValidator.ts)

***

This documentation was generated from [`TupleValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/TupleValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
