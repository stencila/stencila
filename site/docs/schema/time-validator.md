---
title: Time Validator
description: A validator specifying the constraints on a time.
---

# Properties

The `TimeValidator` type has these properties:

| Name      | Description                           | Type                    | Inherited from          |
| --------- | ------------------------------------- | ----------------------- | ----------------------- |
| `id`      | The identifier for this item.         | [`String`](./string.md) | [`Entity`](./entity.md) |
| `minimum` | The inclusive lower limit for a time. | [`Time`](./time.md)     | -                       |
| `maximum` | The inclusive upper limit for a time. | [`Time`](./time.md)     | -                       |

# Related

The `TimeValidator` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `TimeValidator` type is represented in:

- [JSON-LD](https://stencila.org/TimeValidator.jsonld)
- [JSON Schema](https://stencila.org/TimeValidator.schema.json)
- Python class [`TimeValidator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/time_validator.py)
- Rust struct [`TimeValidator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/time_validator.rs)
- TypeScript class [`TimeValidator`](https://github.com/stencila/stencila/blob/main/ts/src/types/TimeValidator.ts)

# Source

This documentation was generated from [`TimeValidator.yaml`](https://github.com/stencila/stencila/blob/main/schema/TimeValidator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
