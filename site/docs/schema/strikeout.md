---
title: Strikeout
description: Content that is marked as struck out.
---

Analogues of `Strikeout` in other schema include:
  - HTML [`<del>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del)
  - JATS XML [`<strike>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/strike.html)
  - MDAST [`Delete`](https://github.com/syntax-tree/mdast#delete)
  - Pandoc [`Strikeout`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L258)
Supersedes the `Delete` inline content type (the name "Strikeout" is less ambiguous than "Delete").


# Properties

The `Strikeout` type has these properties:

| Name      | Description                   | Type                     | Inherited from          |
| --------- | ----------------------------- | ------------------------ | ----------------------- |
| `id`      | The identifier for this item. | [`String`](./string.md)  | [`Entity`](./entity.md) |
| `content` | The content that is marked.   | [`Inline`](./inline.md)* | [`Mark`](./mark.md)     |

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
