---
title: Constant Validator
description: A validator specifying a constant value that a node must have.
---

This is a type used in Stencila Schema for validating nodes against a fixed value.

It adapts the JSON Schema
[`const`](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.3)
concept to Stencila's validator system, allowing a constraint to require exact
equality with a single node value.

The main property is `value`.


# Properties

The `ConstantValidator` type has these properties:

| Name    | Description                        | Type                    | Inherited from          |
| ------- | ---------------------------------- | ----------------------- | ----------------------- |
| `value` | The value that the node must have. | [`Node`](./node.md)     | -                       |
| `id`    | The identifier for this item.      | [`String`](./string.md) | [`Entity`](./entity.md) |

# Related

The `ConstantValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ConstantValidator` type is represented in:

- [JSON-LD](https://stencila.org/ConstantValidator.jsonld)
- [JSON Schema](https://stencila.org/ConstantValidator.schema.json)
- Python class [`ConstantValidator`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ConstantValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/constant_validator.rs)
- TypeScript class [`ConstantValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConstantValidator.ts)

***

This documentation was generated from [`ConstantValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConstantValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
