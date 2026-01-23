---
title: Duration
description: A value that represents the difference between two timestamps.
---

# Properties

The `Duration` type has these properties:

| Name       | Description                                | Type                         | Inherited from          |
| ---------- | ------------------------------------------ | ---------------------------- | ----------------------- |
| `id`       | The identifier for this item.              | [`String`](./string.md)      | [`Entity`](./entity.md) |
| `value`    | The time difference in `timeUnit`s.        | [`Integer`](./integer.md)    | -                       |
| `timeUnit` | The time unit that the `value` represents. | [`TimeUnit`](./time-unit.md) | -                       |

# Related

The `Duration` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Duration` type is represented in:

- [JSON-LD](https://stencila.org/Duration.jsonld)
- [JSON Schema](https://stencila.org/Duration.schema.json)
- Python class [`Duration`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Duration`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/duration.rs)
- TypeScript class [`Duration`](https://github.com/stencila/stencila/blob/main/ts/src/types/Duration.ts)

***

This documentation was generated from [`Duration.yaml`](https://github.com/stencila/stencila/blob/main/schema/Duration.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
