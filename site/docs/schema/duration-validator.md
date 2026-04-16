---
title: Duration Validator
description: A validator specifying the constraints on a duration.
---

This is a type used in Stencila Schema for validating [`Duration`](./duration.md)
nodes.

It exists so document parameters and structured data can express constraints
over durations in the same typed validator system used for other primitives.
This is useful for execution settings, time limits, and data validation.

Key properties define minimum, maximum, and related duration constraints.


# Properties

The `DurationValidator` type has these properties:

| Name        | Description                                | Type                          | Inherited from          |
| ----------- | ------------------------------------------ | ----------------------------- | ----------------------- |
| `timeUnits` | The time units that the duration can have. | [`TimeUnit`](./time-unit.md)* | -                       |
| `minimum`   | The inclusive lower limit for a duration.  | [`Duration`](./duration.md)   | -                       |
| `maximum`   | The inclusive upper limit for a duration.  | [`Duration`](./duration.md)   | -                       |
| `id`        | The identifier for this item.              | [`String`](./string.md)       | [`Entity`](./entity.md) |

# Related

The `DurationValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `DurationValidator` type is represented in:

- [JSON-LD](https://stencila.org/DurationValidator.jsonld)
- [JSON Schema](https://stencila.org/DurationValidator.schema.json)
- Python class [`DurationValidator`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`DurationValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/duration_validator.rs)
- TypeScript class [`DurationValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/DurationValidator.ts)

***

This documentation was generated from [`DurationValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/DurationValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
