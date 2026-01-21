---
title: Date Time
description: A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.
---

# Properties

The `DateTime` type has these properties:

| Name    | Description                     | Type                    | Inherited from          |
| ------- | ------------------------------- | ----------------------- | ----------------------- |
| `id`    | The identifier for this item.   | [`String`](./string.md) | [`Entity`](./entity.md) |
| `value` | The date as an ISO 8601 string. | [`String`](./string.md) | -                       |

# Related

The `DateTime` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `DateTime` type is represented in:

- [JSON-LD](https://stencila.org/DateTime.jsonld)
- [JSON Schema](https://stencila.org/DateTime.schema.json)
- Python class [`DateTime`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/date_time.py)
- Rust struct [`DateTime`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/date_time.rs)
- TypeScript class [`DateTime`](https://github.com/stencila/stencila/blob/main/ts/src/types/DateTime.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `DateTime` type are generated using the following strategies.

::: table

| Property | Complexity | Description                                                                     | Strategy                                                                                                     |
| -------- | ---------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `value`  | Min+       | Generate a fixed date-time string.                                              | `String::from("2022-02-22T22:22:22")`                                                                        |
|          | Low+       | Generate a random date-time string.                                             | Regex `[0-9]{4}-[01][0-9]-[0-3][0-9]T[0-2][0-9]:[0-5][0-9]:[0-5][0-9]\.[0-9]+([+-][0-2][0-9]:[0-5][0-9]\|Z)` |
|          | High+      | Generate a random string of up to 20 alphanumeric characters, colons & hyphens. | Regex `[a-zA-Z0-9\-:]{1,20}`                                                                                 |
|          | Max        | Generate an arbitrary string.                                                   | `String::arbitrary()`                                                                                        |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`DateTime.yaml`](https://github.com/stencila/stencila/blob/main/schema/DateTime.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
