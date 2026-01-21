---
title: List Item
description: A single item in a list.
---

This is an implementation, and extension, of schema.org [`ListItem`](https://schema.org/ListItem).
It extends schema.ord `ListItem` by adding `content` and `isChecked` properties.

Analogues of `ListItem` in other schema include:
  - JATS XML `<list-item>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/list-item.html)
  - HTML [`<li>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li)
  - MDAST [`ListItem`](https://github.com/syntax-tree/mdast#listitem)
  - OpenDocument [`<text:list-item>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415154_253892949)


# Properties

The `ListItem` type has these properties:

| Name             | Description                                                | Type                                                                 | Inherited from          |
| ---------------- | ---------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `id`             | The identifier for this item.                              | [`String`](./string.md)                                              | [`Entity`](./entity.md) |
| `alternateNames` | Alternate names (aliases) for the item.                    | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`    | A description of the item.                                 | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`    | Any kind of identifier for any kind of Thing.              | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`         | Images of the item.                                        | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`           | The name of the item.                                      | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`            | The URL of the item.                                       | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `content`        | The content of the list item.                              | [`Block`](./block.md)*                                               | -                       |
| `item`           | The item represented by this list item.                    | [`Node`](./node.md)                                                  | -                       |
| `isChecked`      | A flag to indicate if this list item is checked.           | [`Boolean`](./boolean.md)                                            | -                       |
| `position`       | The position of the item in a series or sequence of items. | [`Integer`](./integer.md)                                            | -                       |

# Related

The `ListItem` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Bindings

The `ListItem` type is represented in:

- [JSON-LD](https://stencila.org/ListItem.jsonld)
- [JSON Schema](https://stencila.org/ListItem.schema.json)
- Python class [`ListItem`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/list_item.py)
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

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`ListItem.yaml`](https://github.com/stencila/stencila/blob/main/schema/ListItem.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
