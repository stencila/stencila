---
title: Date
description: A calendar date encoded as a ISO 8601 string.
---

This is an implementation of schema.org [`Date`](https://schema.org/Date).

In Stencila Schema it is represented as a node so dates can be carried through
document, metadata, and execution workflows with explicit typing instead of
plain strings alone. This enables validation, serialization, and rich
rendering while remaining interoperable with schema.org-derived models.

See also related temporal types such as [`DateTime`](./date-time.md),
[`Time`](./time.md), and [`Timestamp`](./timestamp.md).


# Analogues

The following external types, elements, or nodes are similar to a `Date`:

- schema.org [`Date`](https://schema.org/Date)
- JATS [`<date>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/date.html): Close JATS analogue for structured date values in document metadata.

# Properties

The `Date` type has these properties:

| Name    | Description                     | Type                    | Inherited from          |
| ------- | ------------------------------- | ----------------------- | ----------------------- |
| `value` | The date as an ISO 8601 string. | [`String`](./string.md) | -                       |
| `id`    | The identifier for this item.   | [`String`](./string.md) | [`Entity`](./entity.md) |

# Related

The `Date` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Date` type is represented in:

- [JSON-LD](https://stencila.org/Date.jsonld)
- [JSON Schema](https://stencila.org/Date.schema.json)
- Python class [`Date`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Date`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/date.rs)
- TypeScript class [`Date`](https://github.com/stencila/stencila/blob/main/ts/src/types/Date.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Date` type are generated using the following strategies.

::: table

| Property | Complexity | Description                                                             | Strategy                              |
| -------- | ---------- | ----------------------------------------------------------------------- | ------------------------------------- |
| `value`  | Min+       | Generate a fixed date string.                                           | `String::from("2022-02-22")`          |
|          | Low+       | Generate a random date string.                                          | Regex `[0-9]{4}-[01][0-9]-[0-3][1-9]` |
|          | High+      | Generate a random string of up to 10 alphanumeric characters & hyphens. | Regex `[a-zA-Z0-9\-]{1,10}`           |
|          | Max        | Generate an arbitrary string.                                           | `String::arbitrary()`                 |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`Date.yaml`](https://github.com/stencila/stencila/blob/main/schema/Date.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
