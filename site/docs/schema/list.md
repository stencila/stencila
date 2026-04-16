---
title: List
description: A list of items.
---

This is an implementation of schema.org
[`ItemList`](https://schema.org/ItemList), exposed in Stencila Schema as
`List`.

The type is renamed to use more familiar document-model vocabulary, and the
schema.org properties `itemListElement` and `itemListOrder` are correspondingly
exposed as `items` and `order`. In Stencila Schema it is used for ordered,
unordered, and checklist-style document lists with rich item content.

Key properties include `items`, `order`, and optional authorship metadata.


# Analogues

The following external types, elements, or nodes are similar to a `List`:

- HTML [`<ul>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ul): Close analogue for unordered lists; Stencila uses a single `List` type and records list style in `order` rather than separate element types.
- HTML [`<ol>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ol): Close analogue for ordered lists; descending and checklist semantics are represented via `order` and item metadata rather than distinct HTML elements.
- JATS [`<list>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/list.html): Closest JATS container analogue for list content.
- Pandoc [`BulletList`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:BulletList): Close analogue for unordered lists; ordered and checklist forms are represented differently in Pandoc.
- Pandoc [`OrderedList`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:OrderedList): Close analogue for ordered lists; Stencila unifies ordered and unordered lists in one node type.
- MDAST [`List`](https://github.com/syntax-tree/mdast#list): Closest MDAST analogue for list containers, though task-list state is carried on items in Stencila.

# Properties

The `List` type has these properties:

| Name         | Description                                                 | Type                                        | Inherited from          |
| ------------ | ----------------------------------------------------------- | ------------------------------------------- | ----------------------- |
| `items`      | The items in the list.                                      | [`ListItem`](./list-item.md)*               | -                       |
| `order`      | The ordering of the list.                                   | [`ListOrder`](./list-order.md)              | -                       |
| `authors`    | The authors of the list.                                    | [`Author`](./author.md)*                    | -                       |
| `provenance` | A summary of the provenance of the content within the list. | [`ProvenanceCount`](./provenance-count.md)* | -                       |
| `id`         | The identifier for this item.                               | [`String`](./string.md)                     | [`Entity`](./entity.md) |

# Related

The `List` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `List` type is represented in:

- [JSON-LD](https://stencila.org/List.jsonld)
- [JSON Schema](https://stencila.org/List.schema.json)
- Python class [`List`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`List`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/list.rs)
- TypeScript class [`List`](https://github.com/stencila/stencila/blob/main/ts/src/types/List.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `List` type are generated using the following strategies.

::: table

| Property | Complexity | Description                                                | Strategy                                                             |
| -------- | ---------- | ---------------------------------------------------------- | -------------------------------------------------------------------- |
| `items`  | Min+       | Generate a single, arbitrary, list item.                   | `vec(ListItem::arbitrary(), size_range(1..=1))`                      |
|          | Low+       | Generate up to two, arbitrary, list items.                 | `vec(ListItem::arbitrary(), size_range(1..=2))`                      |
|          | High+      | Generate up to four, arbitrary, list items.                | `vec(ListItem::arbitrary(), size_range(1..=4))`                      |
|          | Max        | Generate up to eight, arbitrary, list items.               | `vec(ListItem::arbitrary(), size_range(1..=8))`                      |
| `order`  | Min+       | Always generate an unordered list.                         | `ListOrder::Unordered`                                               |
|          | Low+       | Randomly generate either an unordered, or ascending, list. | `prop_oneof![Just(ListOrder::Unordered),Just(ListOrder::Ascending)]` |
|          | High+      | Generate an arbitrary list ordering.                       | `ListOrder::arbitrary()`                                             |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`List.yaml`](https://github.com/stencila/stencila/blob/main/schema/List.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
