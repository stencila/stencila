---
title: Timestamp
description: A value that represents a point in time.
---

This is a temporal type used in Stencila Schema for exact points in time.

Although analogous to date-time concepts elsewhere, it exists separately in
Stencila Schema because timestamps are heavily used for execution events,
provenance, and document modification tracking. Modeling them explicitly
clarifies their role as event times rather than general-purpose date-time
metadata.

See properties such as `executionEnded`, `lastModified`, and other
timestamp-bearing metadata throughout the schema.


# Analogues

The following external types, elements, or nodes are similar to a `Timestamp`:

- [Unix timestamp](https://en.wikipedia.org/wiki/Unix_time): Closest external analogue, though Stencila stores the epoch offset together with explicit `timeUnit` rather than assuming seconds.

# Properties

The `Timestamp` type has these properties:

| Name       | Description                                                                      | Type                         | Inherited from          |
| ---------- | -------------------------------------------------------------------------------- | ---------------------------- | ----------------------- |
| `value`    | The time, in `timeUnit`s, before or after the Unix Epoch (1970-01-01T00:00:00Z). | [`Integer`](./integer.md)    | -                       |
| `timeUnit` | The time unit that the `value` represents.                                       | [`TimeUnit`](./time-unit.md) | -                       |
| `id`       | The identifier for this item.                                                    | [`String`](./string.md)      | [`Entity`](./entity.md) |

# Related

The `Timestamp` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Timestamp` type is represented in:

- [JSON-LD](https://stencila.org/Timestamp.jsonld)
- [JSON Schema](https://stencila.org/Timestamp.schema.json)
- Python class [`Timestamp`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Timestamp`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/timestamp.rs)
- TypeScript class [`Timestamp`](https://github.com/stencila/stencila/blob/main/ts/src/types/Timestamp.ts)

***

This documentation was generated from [`Timestamp.yaml`](https://github.com/stencila/stencila/blob/main/schema/Timestamp.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
