---
title: Underline
description: Inline text that is underlined.
---

Analogues of `Underline` in other schema include:
- Pandoc [`Underline`](https://github.com/jgm/pandoc-types/blob/master/src/Text/Pandoc/Definition.hs)


# Properties

The `Underline` type has these properties:

| Name      | Description                   | Type                     | Inherited from          |
| --------- | ----------------------------- | ------------------------ | ----------------------- |
| `id`      | The identifier for this item. | [`String`](./string.md)  | [`Entity`](./entity.md) |
| `content` | The content that is marked.   | [`Inline`](./inline.md)* | [`Mark`](./mark.md)     |

# Related

The `Underline` type is related to these types:

- Parents: [`Mark`](./mark.md)
- Children: none

# Bindings

The `Underline` type is represented in:

- [JSON-LD](https://stencila.org/Underline.jsonld)
- [JSON Schema](https://stencila.org/Underline.schema.json)
- Python class [`Underline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/underline.py)
- Rust struct [`Underline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/underline.rs)
- TypeScript class [`Underline`](https://github.com/stencila/stencila/blob/main/ts/src/types/Underline.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Underline` type are generated using the following strategies.

::: table

| Property  | Complexity | Description                                                | Strategy                       |
| --------- | ---------- | ---------------------------------------------------------- | ------------------------------ |
| `content` | Min+       | Generate a single fixed text value.                        | `vec![t("text")]`              |
|           | Low+       | Generate a single arbitrary, non-recursive, inline node    | `vec_inlines_non_recursive(1)` |
|           | High+      | Generate up to two arbitrary, non-recursive, inline nodes  | `vec_inlines_non_recursive(2)` |
|           | Max        | Generate up to four arbitrary, non-recursive, inline nodes | `vec_inlines_non_recursive(4)` |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`Underline.yaml`](https://github.com/stencila/stencila/blob/main/schema/Underline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
