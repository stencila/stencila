---
title: Enum Validator
description: A schema specifying that a node must be one of several values.
---

Analogous to the JSON Schema [`enum` keyword](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.2).

# Properties

The `EnumValidator` type has these properties:

| Name     | Description                                            | Type                    | Inherited from          |
| -------- | ------------------------------------------------------ | ----------------------- | ----------------------- |
| `id`     | The identifier for this item.                          | [`String`](./string.md) | [`Entity`](./entity.md) |
| `values` | A node is valid if it is equal to any of these values. | [`Node`](./node.md)*    | -                       |

# Related

The `EnumValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `EnumValidator` type is represented in:

- [JSON-LD](https://stencila.org/EnumValidator.jsonld)
- [JSON Schema](https://stencila.org/EnumValidator.schema.json)
- Python class [`EnumValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/enum_validator.py)
- Rust struct [`EnumValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/enum_validator.rs)
- TypeScript class [`EnumValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/EnumValidator.ts)

# Source

This documentation was generated from [`EnumValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/EnumValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
