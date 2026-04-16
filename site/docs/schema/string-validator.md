---
title: String Validator
description: A validator for string values.
---

This is a type used in Stencila Schema for validating [`String`](./string.md)
nodes.

It adapts common JSON Schema string constraints to Stencila's validator
system, allowing length limits and pattern checks to be expressed as
structured validator nodes.

Key properties include `minLength`, `maxLength`, and `pattern`.


# Properties

The `StringValidator` type has these properties:

| Name        | Description                                         | Type                      | Inherited from          |
| ----------- | --------------------------------------------------- | ------------------------- | ----------------------- |
| `minLength` | The minimum length for a string node.               | [`Integer`](./integer.md) | -                       |
| `maxLength` | The maximum length for a string node.               | [`Integer`](./integer.md) | -                       |
| `pattern`   | A regular expression that a string node must match. | [`String`](./string.md)   | -                       |
| `id`        | The identifier for this item.                       | [`String`](./string.md)   | [`Entity`](./entity.md) |

# Related

The `StringValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `StringValidator` type is represented in:

- [JSON-LD](https://stencila.org/StringValidator.jsonld)
- [JSON Schema](https://stencila.org/StringValidator.schema.json)
- Python class [`StringValidator`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`StringValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/string_validator.rs)
- TypeScript class [`StringValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/StringValidator.ts)

***

This documentation was generated from [`StringValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/StringValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
