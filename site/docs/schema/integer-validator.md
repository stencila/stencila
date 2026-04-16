---
title: Integer Validator
description: A validator for integer values.
---

This is a type used in Stencila Schema for validating [`Integer`](./integer.md)
nodes.

It adapts the JSON Schema integer type concept to Stencila's validator
system, reusing the numeric range and divisibility constraints defined by
[`NumberValidator`](./number-validator.md) while additionally requiring the
value to have no fractional part.

Key constraints are inherited from [`NumberValidator`](./number-validator.md).


# Properties

The `IntegerValidator` type has these properties:

| Name               | Description                                         | Type                    | Inherited from                             |
| ------------------ | --------------------------------------------------- | ----------------------- | ------------------------------------------ |
| `minimum`          | The inclusive lower limit for a numeric node.       | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) |
| `exclusiveMinimum` | The exclusive lower limit for a numeric node.       | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) |
| `maximum`          | The inclusive upper limit for a numeric node.       | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) |
| `exclusiveMaximum` | The exclusive upper limit for a numeric node.       | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) |
| `multipleOf`       | A number that a numeric node must be a multiple of. | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) |
| `id`               | The identifier for this item.                       | [`String`](./string.md) | [`Entity`](./entity.md)                    |

# Related

The `IntegerValidator` type is related to these types:

- Parents: [`NumberValidator`](./number-validator.md)
- Children: none

# Bindings

The `IntegerValidator` type is represented in:

- [JSON-LD](https://stencila.org/IntegerValidator.jsonld)
- [JSON Schema](https://stencila.org/IntegerValidator.schema.json)
- Python class [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/integer_validator.rs)
- TypeScript class [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/IntegerValidator.ts)

***

This documentation was generated from [`IntegerValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/IntegerValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
