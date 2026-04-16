---
title: List Item
description: A single item in a list.
---

This is an implementation of schema.org
[`ListItem`](https://schema.org/ListItem), extended in Stencila Schema for
document-oriented list content.

In addition to the schema.org model, it supports block `content` for rich list
items and `isChecked` for task-list semantics. This makes it suitable for
ordinary prose lists as well as checklists and mixed-content document lists.

Key properties include `content`, `item`, `isChecked`, and `position`.


# Analogues

The following external types, elements, or nodes are similar to a `ListItem`:

- schema.org [`ListItem`](https://schema.org/ListItem): Direct schema.org source type, extended in Stencila with rich block `content` and task-list state.
- HTML [`<li>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li): Closest HTML analogue for list items, though Stencila can also carry `isChecked`, `position`, and structured node-valued `item` metadata.
- JATS [`<list-item>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/list-item.html): Closest JATS analogue for list items.
- Pandoc [`Plain`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Plain): One common Pandoc representation for a simple list item containing inline-only content.
- Pandoc [`BulletList`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:BulletList): Pandoc models list items as nested block lists within list constructors rather than as standalone `ListItem` nodes.
- MDAST [`ListItem`](https://github.com/syntax-tree/mdast#listitem): Closest MDAST analogue; Stencila additionally supports schema.org-style item metadata and explicit checklist state.

# Properties

The `ListItem` type has these properties:

| Name             | Description                                                | Type                                                                 | Inherited from          |
| ---------------- | ---------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `content`        | The content of the list item.                              | [`Block`](./block.md)*                                               | -                       |
| `item`           | The item represented by this list item.                    | [`Node`](./node.md)                                                  | -                       |
| `isChecked`      | A flag to indicate if this list item is checked.           | [`Boolean`](./boolean.md)                                            | -                       |
| `position`       | The position of the item in a series or sequence of items. | [`Integer`](./integer.md)                                            | -                       |
| `alternateNames` | Alternate names (aliases) for the item.                    | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`    | A description of the item.                                 | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`    | Any kind of identifier for any kind of Thing.              | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`         | Images of the item.                                        | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`           | The name of the item.                                      | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`            | The URL of the item.                                       | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `id`             | The identifier for this item.                              | [`String`](./string.md)                                              | [`Entity`](./entity.md) |

# Related

The `ListItem` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Bindings

The `ListItem` type is represented in:

- [JSON-LD](https://stencila.org/ListItem.jsonld)
- [JSON Schema](https://stencila.org/ListItem.schema.json)
- Python class [`ListItem`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ListItem`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/list_item.rs)
- TypeScript class [`ListItem`](https://github.com/stencila/stencila/blob/main/ts/src/types/ListItem.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `ListItem` type are generated using the following strategies.

::: table

| Property  | Complexity | Description                                     | Strategy                  |
| --------- | ---------- | ----------------------------------------------- | ------------------------- |
| `content` | Min+       | Generate a single, arbitrary, paragraph         | `vec_paragraphs(1)`       |
|           | Low+       | Generate one, arbitrary, non-list block         | `vec_blocks_list_item(1)` |
|           | High+      | Generate up to two, arbitrary, non-list blocks  | `vec_blocks_list_item(2)` |
|           | Max        | Generate up to four, arbitrary, non-list blocks | `vec_blocks_list_item(4)` |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`ListItem.yaml`](https://github.com/stencila/stencila/blob/main/schema/ListItem.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
