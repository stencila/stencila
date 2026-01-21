---
title: Duration Validator
description: A validator specifying the constraints on a duration.
---

# Properties

The `DurationValidator` type has these properties:

| Name        | Description                                | Type                          | Inherited from          |
| ----------- | ------------------------------------------ | ----------------------------- | ----------------------- |
| `id`        | The identifier for this item.              | [`String`](./string.md)       | [`Entity`](./entity.md) |
| `timeUnits` | The time units that the duration can have. | [`TimeUnit`](./time-unit.md)* | -                       |
| `minimum`   | The inclusive lower limit for a duration.  | [`Duration`](./duration.md)   | -                       |
| `maximum`   | The inclusive upper limit for a duration.  | [`Duration`](./duration.md)   | -                       |

# Related

The `DurationValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `DurationValidator` type is represented in:

- [JSON-LD](https://stencila.org/DurationValidator.jsonld)
- [JSON Schema](https://stencila.org/DurationValidator.schema.json)
- Python class [`DurationValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/duration_validator.py)
- Rust struct [`DurationValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/duration_validator.rs)
- TypeScript class [`DurationValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/DurationValidator.ts)

# Source

This documentation was generated from [`DurationValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/DurationValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
