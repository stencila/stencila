---
title: Date Time Validator
description: A validator specifying the constraints on a date-time.
---

# Properties

The `DateTimeValidator` type has these properties:

| Name      | Description                                | Type                         | Inherited from          |
| --------- | ------------------------------------------ | ---------------------------- | ----------------------- |
| `id`      | The identifier for this item.              | [`String`](./string.md)      | [`Entity`](./entity.md) |
| `minimum` | The inclusive lower limit for a date-time. | [`DateTime`](./date-time.md) | -                       |
| `maximum` | The inclusive upper limit for a date-time. | [`DateTime`](./date-time.md) | -                       |

# Related

The `DateTimeValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `DateTimeValidator` type is represented in:

- [JSON-LD](https://stencila.org/DateTimeValidator.jsonld)
- [JSON Schema](https://stencila.org/DateTimeValidator.schema.json)
- Python class [`DateTimeValidator`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`DateTimeValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/date_time_validator.rs)
- TypeScript class [`DateTimeValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/DateTimeValidator.ts)

***

This documentation was generated from [`DateTimeValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/DateTimeValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
