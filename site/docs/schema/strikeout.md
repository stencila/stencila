---
title: Strikeout
description: Content that is marked as struck out.
---

This is a Stencila-native inline mark for struck-out text, with close
analogues in HTML
[`<del>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del), JATS
[`<strike>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/strike.html),
MDAST [`Delete`](https://github.com/syntax-tree/mdast#delete), and Pandoc
[`Strikeout`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L258).

In Stencila Schema it is used as a presentational and editorial inline mark
for text that should render with a strike-through. It supersedes the older
`Delete` inline type because `Strikeout` more clearly describes the mark itself
rather than an editing action.

Key semantics are inherited from [`Mark`](./mark.md), with this type chiefly
signaling strike-through rendering for its `content`.


# Analogues

The following external types, elements, or nodes are similar to a `Strikeout`:

- HTML [`<del>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del): Close HTML analogue for struck-through text, though Stencila `Strikeout` is used as a general strike-through mark rather than specifically representing deletions.
- JATS [`<strike>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/strike.html)
- Pandoc [`Strikeout`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Strikeout)
- MDAST [`Delete`](https://github.com/syntax-tree/mdast#delete): Closest MDAST analogue, although the MDAST name emphasizes deletion semantics rather than the visual mark.

# Properties

The `Strikeout` type has these properties:

| Name      | Description                   | Type                     | Inherited from          |
| --------- | ----------------------------- | ------------------------ | ----------------------- |
| `content` | The content that is marked.   | [`Inline`](./inline.md)* | [`Mark`](./mark.md)     |
| `id`      | The identifier for this item. | [`String`](./string.md)  | [`Entity`](./entity.md) |

# Related

The `Strikeout` type is related to these types:

- Parents: [`Mark`](./mark.md)
- Children: none

# Bindings

The `Strikeout` type is represented in:

- [JSON-LD](https://stencila.org/Strikeout.jsonld)
- [JSON Schema](https://stencila.org/Strikeout.schema.json)
- Python class [`Strikeout`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Strikeout`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/strikeout.rs)
- TypeScript class [`Strikeout`](https://github.com/stencila/stencila/blob/main/ts/src/types/Strikeout.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Strikeout` type are generated using the following strategies.

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

This documentation was generated from [`Strikeout.yaml`](https://github.com/stencila/stencila/blob/main/schema/Strikeout.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
