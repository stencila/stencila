---
title: Function
description: A function signature.
---

This is a type used in Stencila Schema for describing callable functions in
documents and execution contexts.

It exists to represent function signatures structurally, including parameters
and return types, so functions can be documented, exposed to tools, and
invoked from executable nodes. This helps bridge document content with
computational behavior.

Key properties include the function `name`, `parameters`, and return-value
description.


This type is marked as experimental and is likely to change.

# Analogues

The following external types, elements, or nodes are similar to a `Function`:

- [OpenAPI operation signature](https://spec.openapis.org/oas/latest.html): Approximate analogue for structured callable signatures with parameters and return-like schemas, though Stencila `Function` is language-agnostic and document-oriented.

# Properties

The `Function` type has these properties:

| Name         | Description                      | Type                           | Inherited from          |
| ------------ | -------------------------------- | ------------------------------ | ----------------------- |
| `name`       | The name of the function.        | [`String`](./string.md)        | -                       |
| `parameters` | The parameters of the function.  | [`Parameter`](./parameter.md)* | -                       |
| `returns`    | The return type of the function. | [`Validator`](./validator.md)  | -                       |
| `id`         | The identifier for this item.    | [`String`](./string.md)        | [`Entity`](./entity.md) |

# Related

The `Function` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Function` type is represented in:

- [JSON-LD](https://stencila.org/Function.jsonld)
- [JSON Schema](https://stencila.org/Function.schema.json)
- Python class [`Function`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Function`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/function.rs)
- TypeScript class [`Function`](https://github.com/stencila/stencila/blob/main/ts/src/types/Function.ts)

***

This documentation was generated from [`Function.yaml`](https://github.com/stencila/stencila/blob/main/schema/Function.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
