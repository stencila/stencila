---
title: Timestamp Validator
description: A validator specifying the constraints on a timestamp.
---

# Properties

The `TimestampValidator` type has these properties:

| Name        | Description                                 | Type                          | Inherited from          |
| ----------- | ------------------------------------------- | ----------------------------- | ----------------------- |
| `id`        | The identifier for this item.               | [`String`](./string.md)       | [`Entity`](./entity.md) |
| `timeUnits` | The time units that the timestamp can have. | [`TimeUnit`](./time-unit.md)* | -                       |
| `minimum`   | The inclusive lower limit for a timestamp.  | [`Timestamp`](./timestamp.md) | -                       |
| `maximum`   | The inclusive upper limit for a timestamp.  | [`Timestamp`](./timestamp.md) | -                       |

# Related

The `TimestampValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `TimestampValidator` type is represented in:

- [JSON-LD](https://stencila.org/TimestampValidator.jsonld)
- [JSON Schema](https://stencila.org/TimestampValidator.schema.json)
- Python class [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/timestamp_validator.py)
- Rust struct [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/timestamp_validator.rs)
- TypeScript class [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/TimestampValidator.ts)

# Source

This documentation was generated from [`TimestampValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/TimestampValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
