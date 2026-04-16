---
title: Enum Validator
description: A validator for a fixed set of allowed values.
---

This is a type used in Stencila Schema for validating nodes against an enumerated
set of values.

It adapts the JSON Schema
[`enum`](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.2)
concept to Stencila's validator system, allowing a value to be constrained to
one of several explicitly listed node values.

The main property is `values`.


# Properties

The `EnumValidator` type has these properties:

| Name     | Description                                            | Type                    | Inherited from          |
| -------- | ------------------------------------------------------ | ----------------------- | ----------------------- |
| `values` | A node is valid if it is equal to any of these values. | [`Node`](./node.md)*    | -                       |
| `id`     | The identifier for this item.                          | [`String`](./string.md) | [`Entity`](./entity.md) |

# Related

The `EnumValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `EnumValidator` type is represented in:

- [JSON-LD](https://stencila.org/EnumValidator.jsonld)
- [JSON Schema](https://stencila.org/EnumValidator.schema.json)
- Python class [`EnumValidator`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`EnumValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/enum_validator.rs)
- TypeScript class [`EnumValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/EnumValidator.ts)

***

This documentation was generated from [`EnumValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/EnumValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
