---
title: Constant Validator
description: A validator specifying a constant value that a node must have.
---

A node will be valid against this validator if it is equal to the
`value` property. Analogous to the JSON Schema [`const`](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.3) keyword.


# Properties

The `ConstantValidator` type has these properties:

| Name    | Description                        | Type                    | Inherited from          |
| ------- | ---------------------------------- | ----------------------- | ----------------------- |
| `id`    | The identifier for this item.      | [`String`](./string.md) | [`Entity`](./entity.md) |
| `value` | The value that the node must have. | [`Node`](./node.md)     | -                       |

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
