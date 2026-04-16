---
title: Date Validator
description: A validator specifying the constraints on a date.
---

This is a type used in Stencila Schema for validating [`Date`](./date.md) nodes.

It exists so Stencila documents and forms can express constraints over
calendar dates as structured data rather than leaving validation to ad hoc
string parsing. This keeps date validation aligned with the rest of the
validator system.

Key properties define allowable date ranges and related constraints.


# Properties

The `DateValidator` type has these properties:

| Name      | Description                           | Type                    | Inherited from          |
| --------- | ------------------------------------- | ----------------------- | ----------------------- |
| `minimum` | The inclusive lower limit for a date. | [`Date`](./date.md)     | -                       |
| `maximum` | The inclusive upper limit for a date. | [`Date`](./date.md)     | -                       |
| `id`      | The identifier for this item.         | [`String`](./string.md) | [`Entity`](./entity.md) |

# Related

The `DateValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `DateValidator` type is represented in:

- [JSON-LD](https://stencila.org/DateValidator.jsonld)
- [JSON Schema](https://stencila.org/DateValidator.schema.json)
- Python class [`DateValidator`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`DateValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/date_validator.rs)
- TypeScript class [`DateValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/DateValidator.ts)

***

This documentation was generated from [`DateValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/DateValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
