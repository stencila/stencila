---
title: Number Validator
description: A validator for numeric values.
---

This is a type used in Stencila Schema for validating [`Number`](./number.md)
nodes.

It adapts common JSON Schema numeric constraints to Stencila's validator
system, allowing numeric ranges and divisibility rules to be expressed as
structured validator nodes.

Key properties include `minimum`, `exclusiveMinimum`, `maximum`,
`exclusiveMaximum`, and `multipleOf`. See also
[`IntegerValidator`](./integer-validator.md).


# Properties

The `NumberValidator` type has these properties:

| Name               | Description                                         | Type                    | Inherited from          |
| ------------------ | --------------------------------------------------- | ----------------------- | ----------------------- |
| `minimum`          | The inclusive lower limit for a numeric node.       | [`Number`](./number.md) | -                       |
| `exclusiveMinimum` | The exclusive lower limit for a numeric node.       | [`Number`](./number.md) | -                       |
| `maximum`          | The inclusive upper limit for a numeric node.       | [`Number`](./number.md) | -                       |
| `exclusiveMaximum` | The exclusive upper limit for a numeric node.       | [`Number`](./number.md) | -                       |
| `multipleOf`       | A number that a numeric node must be a multiple of. | [`Number`](./number.md) | -                       |
| `id`               | The identifier for this item.                       | [`String`](./string.md) | [`Entity`](./entity.md) |

# Related

The `NumberValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: [`IntegerValidator`](./integer-validator.md)

# Bindings

The `NumberValidator` type is represented in:

- [JSON-LD](https://stencila.org/NumberValidator.jsonld)
- [JSON Schema](https://stencila.org/NumberValidator.schema.json)
- Python class [`NumberValidator`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`NumberValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/number_validator.rs)
- TypeScript class [`NumberValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/NumberValidator.ts)

***

This documentation was generated from [`NumberValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/NumberValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
