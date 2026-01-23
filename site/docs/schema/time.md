---
title: Time
description: A point in time recurring on multiple days.
---

# Properties

The `Time` type has these properties:

| Name    | Description                                                       | Type                    | Inherited from          |
| ------- | ----------------------------------------------------------------- | ----------------------- | ----------------------- |
| `id`    | The identifier for this item.                                     | [`String`](./string.md) | [`Entity`](./entity.md) |
| `value` | The time of day as a string in format `hh:mm:ss[Z\|(+\|-)hh:mm]`. | [`String`](./string.md) | -                       |

# Related

The `Time` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Time` type is represented in:

- [JSON-LD](https://stencila.org/Time.jsonld)
- [JSON Schema](https://stencila.org/Time.schema.json)
- Python class [`Time`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Time`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/time.rs)
- TypeScript class [`Time`](https://github.com/stencila/stencila/blob/main/ts/src/types/Time.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Time` type are generated using the following strategies.

::: table

| Property | Complexity | Description                                                                     | Strategy                                                                       |
| -------- | ---------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `value`  | Min+       | Generate a fixed date-time string.                                              | `String::from("2022-02-22T22:22:22")`                                          |
|          | Low+       | Generate a random date-time string.                                             | Regex `[0-2][0-9]:[0-5][0-9]:[0-5][0-9]\.[0-9]+([+-][0-2][0-9]:[0-5][0-9]\|Z)` |
|          | High+      | Generate a random string of up to 20 alphanumeric characters, colons & hyphens. | Regex `[a-zA-Z0-9\-:]{1,20}`                                                   |
|          | Max        | Generate an arbitrary string.                                                   | `String::arbitrary()`                                                          |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`Time.yaml`](https://github.com/stencila/stencila/blob/main/schema/Time.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
