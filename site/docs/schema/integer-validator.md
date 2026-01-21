---
title: Integer Validator
description: A validator specifying the constraints on an integer node.
---

A node will be valid if it is a number with no fractional part and meets any additional constraints,
such as `multipleOf`, specified in the validator.
Analogous to the JSON Schema `integer` validation [type](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.1.1).


# Properties

The `IntegerValidator` type has these properties:

| Name               | Description                                         | Type                    | Inherited from                             |
| ------------------ | --------------------------------------------------- | ----------------------- | ------------------------------------------ |
| `id`               | The identifier for this item.                       | [`String`](./string.md) | [`Entity`](./entity.md)                    |
| `minimum`          | The inclusive lower limit for a numeric node.       | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) |
| `exclusiveMinimum` | The exclusive lower limit for a numeric node.       | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) |
| `maximum`          | The inclusive upper limit for a numeric node.       | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) |
| `exclusiveMaximum` | The exclusive upper limit for a numeric node.       | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) |
| `multipleOf`       | A number that a numeric node must be a multiple of. | [`Number`](./number.md) | [`NumberValidator`](./number-validator.md) |

# Related

The `IntegerValidator` type is related to these types:

- Parents: [`NumberValidator`](./number-validator.md)
- Children: none

# Bindings

The `IntegerValidator` type is represented in:

- [JSON-LD](https://stencila.org/IntegerValidator.jsonld)
- [JSON Schema](https://stencila.org/IntegerValidator.schema.json)
- Python class [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/integer_validator.py)
- Rust struct [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/integer_validator.rs)
- TypeScript class [`IntegerValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/IntegerValidator.ts)

# Source

This documentation was generated from [`IntegerValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/IntegerValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
