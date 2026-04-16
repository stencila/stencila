---
title: Subscript
description: Subscripted content.
---

This is an inline mark type used in Stencila Schema for subscripted content.

It extends [`Mark`](./mark.md) to preserve subscript semantics in a
format-independent way, which is especially useful in scientific and
mathematical writing.

Key properties are inherited from [`Mark`](./mark.md), especially the wrapped
inline `content`.


# Analogues

The following external types, elements, or nodes are similar to a `Subscript`:

- HTML [`<sub>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sub)
- JATS [`<sub>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/sub.html)
- Pandoc [`Subscript`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Subscript)

# Properties

The `Subscript` type has these properties:

| Name      | Description                   | Type                     | Inherited from          |
| --------- | ----------------------------- | ------------------------ | ----------------------- |
| `content` | The content that is marked.   | [`Inline`](./inline.md)* | [`Mark`](./mark.md)     |
| `id`      | The identifier for this item. | [`String`](./string.md)  | [`Entity`](./entity.md) |

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
