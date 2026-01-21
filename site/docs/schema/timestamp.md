---
title: Timestamp
description: A value that represents a point in time.
---

# Properties

The `Timestamp` type has these properties:

| Name       | Description                                                                      | Type                         | Inherited from          |
| ---------- | -------------------------------------------------------------------------------- | ---------------------------- | ----------------------- |
| `id`       | The identifier for this item.                                                    | [`String`](./string.md)      | [`Entity`](./entity.md) |
| `value`    | The time, in `timeUnit`s, before or after the Unix Epoch (1970-01-01T00:00:00Z). | [`Integer`](./integer.md)    | -                       |
| `timeUnit` | The time unit that the `value` represents.                                       | [`TimeUnit`](./time-unit.md) | -                       |

# Related

The `Timestamp` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Timestamp` type is represented in:

- [JSON-LD](https://stencila.org/Timestamp.jsonld)
- [JSON Schema](https://stencila.org/Timestamp.schema.json)
- Python class [`Timestamp`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/timestamp.py)
- Rust struct [`Timestamp`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/timestamp.rs)
- TypeScript class [`Timestamp`](https://github.com/stencila/stencila/blob/main/ts/src/types/Timestamp.ts)

# Source

This documentation was generated from [`Timestamp.yaml`](https://github.com/stencila/stencila/blob/main/schema/Timestamp.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
