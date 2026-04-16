---
title: Emphasis
description: Emphasized content.
---

This is an inline mark type used in Stencila Schema for emphasis.

It extends [`Mark`](./mark.md) to preserve semantic emphasis in a structured,
format-independent inline model, with analogues in HTML `<em>` and Markdown
emphasis.

Key properties are inherited from [`Mark`](./mark.md), especially the wrapped
inline `content`.


# Analogues

The following external types, elements, or nodes are similar to a `Emphasis`:

- HTML [`<em>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em)
- JATS [`<italic>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/italic.html): Close JATS rendering analogue for emphasized content.
- Pandoc [`Emph`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Emph)
- MDAST [`Emphasis`](https://github.com/syntax-tree/mdast#emphasis)

# Properties

The `Emphasis` type has these properties:

| Name      | Description                   | Type                     | Inherited from          |
| --------- | ----------------------------- | ------------------------ | ----------------------- |
| `content` | The content that is marked.   | [`Inline`](./inline.md)* | [`Mark`](./mark.md)     |
| `id`      | The identifier for this item. | [`String`](./string.md)  | [`Entity`](./entity.md) |

# Related

The `Emphasis` type is related to these types:

- Parents: [`Mark`](./mark.md)
- Children: none

# Bindings

The `Emphasis` type is represented in:

- [JSON-LD](https://stencila.org/Emphasis.jsonld)
- [JSON Schema](https://stencila.org/Emphasis.schema.json)
- Python class [`Emphasis`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Emphasis`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/emphasis.rs)
- TypeScript class [`Emphasis`](https://github.com/stencila/stencila/blob/main/ts/src/types/Emphasis.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Emphasis` type are generated using the following strategies.

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

This documentation was generated from [`Emphasis.yaml`](https://github.com/stencila/stencila/blob/main/schema/Emphasis.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
