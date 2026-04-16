---
title: Duration
description: A value that represents the difference between two timestamps.
---

This is an implementation of schema.org
[`Duration`](https://schema.org/Duration).

In Stencila Schema it is represented as a typed node because durations are
commonly produced by execution, attached to metadata, and rendered in
documents. Modeling them explicitly allows validation and serialization
without losing the distinction between a duration and a timestamp.

See also temporal types such as [`Time`](./time.md) and
[`Timestamp`](./timestamp.md).


# Analogues

The following external types, elements, or nodes are similar to a `Duration`:

- schema.org [`Duration`](https://schema.org/Duration)
- [ISO 8601 duration](https://en.wikipedia.org/wiki/ISO_8601#Durations): Close external duration analogue, though Stencila stores duration as numeric `value` plus explicit `timeUnit` rather than a single encoded string.

# Properties

The `Duration` type has these properties:

| Name       | Description                                | Type                         | Inherited from          |
| ---------- | ------------------------------------------ | ---------------------------- | ----------------------- |
| `value`    | The time difference in `timeUnit`s.        | [`Integer`](./integer.md)    | -                       |
| `timeUnit` | The time unit that the `value` represents. | [`TimeUnit`](./time-unit.md) | -                       |
| `id`       | The identifier for this item.              | [`String`](./string.md)      | [`Entity`](./entity.md) |

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
