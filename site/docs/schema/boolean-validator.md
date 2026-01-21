---
title: Boolean Validator
description: A schema specifying that a node must be a boolean value.
---

A node will be valid against this schema if it is either true or false.
Analogous to the JSON Schema `boolean` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).


# Properties

The `BooleanValidator` type has these properties:

| Name | Description                   | Type                    | Inherited from          |
| ---- | ----------------------------- | ----------------------- | ----------------------- |
| `id` | The identifier for this item. | [`String`](./string.md) | [`Entity`](./entity.md) |

# Related

The `BooleanValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `BooleanValidator` type is represented in:

- [JSON-LD](https://stencila.org/BooleanValidator.jsonld)
- [JSON Schema](https://stencila.org/BooleanValidator.schema.json)
- Python class [`BooleanValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/boolean_validator.py)
- Rust struct [`BooleanValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/boolean_validator.rs)
- TypeScript class [`BooleanValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/BooleanValidator.ts)

# Source

This documentation was generated from [`BooleanValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/BooleanValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
