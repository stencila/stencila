---
title: Function
description: A function with a name, which might take Parameters and return a value of a certain type.
---

This type is marked as experimental and is likely to change.

# Properties

The `Function` type has these properties:

| Name         | Description                      | Type                           | Inherited from          |
| ------------ | -------------------------------- | ------------------------------ | ----------------------- |
| `id`         | The identifier for this item.    | [`String`](./string.md)        | [`Entity`](./entity.md) |
| `name`       | The name of the function.        | [`String`](./string.md)        | -                       |
| `parameters` | The parameters of the function.  | [`Parameter`](./parameter.md)* | -                       |
| `returns`    | The return type of the function. | [`Validator`](./validator.md)  | -                       |

# Related

The `Function` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Function` type is represented in:

- [JSON-LD](https://stencila.org/Function.jsonld)
- [JSON Schema](https://stencila.org/Function.schema.json)
- Python class [`Function`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/function.py)
- Rust struct [`Function`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/function.rs)
- TypeScript class [`Function`](https://github.com/stencila/stencila/blob/main/ts/src/types/Function.ts)

# Source

This documentation was generated from [`Function.yaml`](https://github.com/stencila/stencila/blob/main/schema/Function.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
