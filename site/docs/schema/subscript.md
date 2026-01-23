---
title: Subscript
description: Subscripted content.
---

# Properties

The `Subscript` type has these properties:

| Name      | Description                   | Type                     | Inherited from          |
| --------- | ----------------------------- | ------------------------ | ----------------------- |
| `id`      | The identifier for this item. | [`String`](./string.md)  | [`Entity`](./entity.md) |
| `content` | The content that is marked.   | [`Inline`](./inline.md)* | [`Mark`](./mark.md)     |

# Related

The `Subscript` type is related to these types:

- Parents: [`Mark`](./mark.md)
- Children: none

# Bindings

The `Subscript` type is represented in:

- [JSON-LD](https://stencila.org/Subscript.jsonld)
- [JSON Schema](https://stencila.org/Subscript.schema.json)
- Python class [`Subscript`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Subscript`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/subscript.rs)
- TypeScript class [`Subscript`](https://github.com/stencila/stencila/blob/main/ts/src/types/Subscript.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Subscript` type are generated using the following strategies.

::: table

| Property  | Complexity | Description                                                | Strategy                       |
| --------- | ---------- | ---------------------------------------------------------- | ------------------------------ |
| `content` | Min+       | Generate a single fixed text value.                        | `vec![t("text")]`              |
|           | Low+       | Generate a single arbitrary, non-recursive, inline node    | `vec_inlines_non_recursive(1)` |
|           | High+      | Generate up to two arbitrary, non-recursive, inline nodes  | `vec_inlines_non_recursive(2)` |
|           | Max        | Generate up to four arbitrary, non-recursive, inline nodes | `vec_inlines_non_recursive(4)` |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`Subscript.yaml`](https://github.com/stencila/stencila/blob/main/schema/Subscript.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
