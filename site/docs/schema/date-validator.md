---
title: Date Validator
description: A validator specifying the constraints on a date.
---

# Properties

The `DateValidator` type has these properties:

| Name      | Description                           | Type                    | Inherited from          |
| --------- | ------------------------------------- | ----------------------- | ----------------------- |
| `id`      | The identifier for this item.         | [`String`](./string.md) | [`Entity`](./entity.md) |
| `minimum` | The inclusive lower limit for a date. | [`Date`](./date.md)     | -                       |
| `maximum` | The inclusive upper limit for a date. | [`Date`](./date.md)     | -                       |

# Related

The `DateValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `DateValidator` type is represented in:

- [JSON-LD](https://stencila.org/DateValidator.jsonld)
- [JSON Schema](https://stencila.org/DateValidator.schema.json)
- Python class [`DateValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/date_validator.py)
- Rust struct [`DateValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/date_validator.rs)
- TypeScript class [`DateValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/DateValidator.ts)

# Source

This documentation was generated from [`DateValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/DateValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
