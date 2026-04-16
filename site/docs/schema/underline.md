---
title: Underline
description: Inline text that is underlined.
---

This is an inline mark type used in Stencila Schema for underlined content.

It extends [`Mark`](./mark.md) to preserve underline semantics in a
structured, format-independent inline model.

Key properties are inherited from [`Mark`](./mark.md), especially the wrapped
inline `content`.


# Analogues

The following external types, elements, or nodes are similar to a `Underline`:

- HTML [`<u>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/u)
- JATS [`<underline>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/underline.html)
- Pandoc [`Underline`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Underline)

# Properties

The `Underline` type has these properties:

| Name      | Description                   | Type                     | Inherited from          |
| --------- | ----------------------------- | ------------------------ | ----------------------- |
| `content` | The content that is marked.   | [`Inline`](./inline.md)* | [`Mark`](./mark.md)     |
| `id`      | The identifier for this item. | [`String`](./string.md)  | [`Entity`](./entity.md) |

# Related

The `Underline` type is related to these types:

- Parents: [`Mark`](./mark.md)
- Children: none

# Bindings

The `Underline` type is represented in:

- [JSON-LD](https://stencila.org/Underline.jsonld)
- [JSON Schema](https://stencila.org/Underline.schema.json)
- Python class [`Underline`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
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

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`Underline.yaml`](https://github.com/stencila/stencila/blob/main/schema/Underline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
