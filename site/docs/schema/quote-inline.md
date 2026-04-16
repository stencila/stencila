---
title: Quote Inline
description: Inline, quoted content.
---

This is an inline mark type used in Stencila Schema for quoted content.

It extends [`Mark`](./mark.md) so short quotations within prose can be
represented structurally and serialized consistently across formats, rather
than relying only on punctuation.

Key properties are inherited from [`Mark`](./mark.md), especially the wrapped
inline `content`.


# Analogues

The following external types, elements, or nodes are similar to a `QuoteInline`:

- HTML [`<q>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/q)
- Pandoc [`Quoted`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Quoted): Closest Pandoc analogue for inline quotations, though Pandoc distinguishes quote kind explicitly.
- [MDAST phrasing quotation](https://github.com/syntax-tree/mdast): There is no dedicated core MDAST inline quote node; quotation is usually represented as plain text punctuation or HTML.

# Properties

The `QuoteInline` type has these properties:

| Name      | Description                   | Type                                               | Inherited from          |
| --------- | ----------------------------- | -------------------------------------------------- | ----------------------- |
| `source`  | The source of the quote.      | [`Citation`](./citation.md) \| [`Text`](./text.md) | -                       |
| `content` | The content that is marked.   | [`Inline`](./inline.md)*                           | [`Mark`](./mark.md)     |
| `id`      | The identifier for this item. | [`String`](./string.md)                            | [`Entity`](./entity.md) |

# Related

The `QuoteInline` type is related to these types:

- Parents: [`Mark`](./mark.md)
- Children: none

# Bindings

The `QuoteInline` type is represented in:

- [JSON-LD](https://stencila.org/QuoteInline.jsonld)
- [JSON Schema](https://stencila.org/QuoteInline.schema.json)
- Python class [`QuoteInline`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
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

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`QuoteInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/QuoteInline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
