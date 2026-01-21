---
title: List
description: A list of items.
---

This is an implementation, and renaming, of schema.org [`ItemList`](https://schema.org/ItemList).
Renaming was done as `List` was considered a more developer friendly alternative. Similarly,
schema.org properties `itemListElement` and `itemListOrder` were renamed to `items` and `order`.
Note that, as with every other such renaming in Stencila Schema, a mapping between names is
defined and it is trivial to save Stencila Schema documents using the schema.org vocabulary if so desired.


# Properties

The `List` type has these properties:

| Name         | Description                                                 | Type                                        | Inherited from          |
| ------------ | ----------------------------------------------------------- | ------------------------------------------- | ----------------------- |
| `id`         | The identifier for this item.                               | [`String`](./string.md)                     | [`Entity`](./entity.md) |
| `items`      | The items in the list.                                      | [`ListItem`](./list-item.md)*               | -                       |
| `order`      | The ordering of the list.                                   | [`ListOrder`](./list-order.md)              | -                       |
| `authors`    | The authors of the list.                                    | [`Author`](./author.md)*                    | -                       |
| `provenance` | A summary of the provenance of the content within the list. | [`ProvenanceCount`](./provenance-count.md)* | -                       |

# Related

The `List` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `List` type is represented in:

- [JSON-LD](https://stencila.org/List.jsonld)
- [JSON Schema](https://stencila.org/List.schema.json)
- Python class [`List`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/list.py)
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

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`List.yaml`](https://github.com/stencila/stencila/blob/main/schema/List.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
