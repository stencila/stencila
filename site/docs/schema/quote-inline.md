---
title: Quote Inline
description: Inline, quoted content.
---

# Properties

The `QuoteInline` type has these properties:

| Name      | Description                   | Type                                               | Inherited from          |
| --------- | ----------------------------- | -------------------------------------------------- | ----------------------- |
| `id`      | The identifier for this item. | [`String`](./string.md)                            | [`Entity`](./entity.md) |
| `content` | The content that is marked.   | [`Inline`](./inline.md)*                           | [`Mark`](./mark.md)     |
| `source`  | The source of the quote.      | [`Citation`](./citation.md) \| [`Text`](./text.md) | -                       |

# Related

The `QuoteInline` type is related to these types:

- Parents: [`Mark`](./mark.md)
- Children: none

# Bindings

The `QuoteInline` type is represented in:

- [JSON-LD](https://stencila.org/QuoteInline.jsonld)
- [JSON Schema](https://stencila.org/QuoteInline.schema.json)
- Python class [`QuoteInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/quote_inline.py)
- Rust struct [`QuoteInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/quote_inline.rs)
- TypeScript class [`QuoteInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/QuoteInline.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `QuoteInline` type are generated using the following strategies.

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

This documentation was generated from [`QuoteInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/QuoteInline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
