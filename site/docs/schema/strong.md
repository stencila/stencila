---
title: Strong
description: Strongly emphasized content.
---

This is an inline mark type used in Stencila Schema for strong emphasis.

It extends [`Mark`](./mark.md), with analogues in HTML `<strong>` and Markdown
strong emphasis, to preserve semantic emphasis in a structured inline model.

Key properties are inherited from [`Mark`](./mark.md), especially the wrapped
inline `content`.


# Analogues

The following external types, elements, or nodes are similar to a `Strong`:

- HTML [`<strong>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/strong)
- JATS [`<bold>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/bold.html): Close JATS rendering analogue for strong emphasis.
- Pandoc [`Strong`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Strong)
- MDAST [`Strong`](https://github.com/syntax-tree/mdast#strong)

# Properties

The `Strong` type has these properties:

| Name      | Description                   | Type                     | Inherited from          |
| --------- | ----------------------------- | ------------------------ | ----------------------- |
| `content` | The content that is marked.   | [`Inline`](./inline.md)* | [`Mark`](./mark.md)     |
| `id`      | The identifier for this item. | [`String`](./string.md)  | [`Entity`](./entity.md) |

# Related

The `Strong` type is related to these types:

- Parents: [`Mark`](./mark.md)
- Children: none

# Bindings

The `Strong` type is represented in:

- [JSON-LD](https://stencila.org/Strong.jsonld)
- [JSON Schema](https://stencila.org/Strong.schema.json)
- Python class [`Strong`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Strong`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/strong.rs)
- TypeScript class [`Strong`](https://github.com/stencila/stencila/blob/main/ts/src/types/Strong.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Strong` type are generated using the following strategies.

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

This documentation was generated from [`Strong.yaml`](https://github.com/stencila/stencila/blob/main/schema/Strong.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
