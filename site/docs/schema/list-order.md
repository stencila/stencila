---
title: List Order
description: The ordering of a list.
---

This is an implementation of schema.org
[`ItemListOrderType`](https://schema.org/ItemListOrderType).

In Stencila Schema it is used by [`List`](./list.md) to represent ordering
semantics in a controlled way that can be mapped to authoring and publishing
formats.

See [`List.order`](./list.md#order) for the property that uses this
enumeration.


# Analogues

The following external types, elements, or nodes are similar to a `ListOrder`:

- schema.org [`ItemListOrderType`](https://schema.org/ItemListOrderType)
- [HTML list type semantics](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ol): Approximate analogue because HTML expresses ordered-versus-unordered distinctions structurally and has limited native support for descending order.

# Members

The `ListOrder` type has these members:

| Member       | Description |
| ------------ | ----------- |
| `Ascending`  | -           |
| `Descending` | -           |
| `Unordered`  | -           |

# Bindings

The `ListOrder` type is represented in:

- [JSON-LD](https://stencila.org/ListOrder.jsonld)
- [JSON Schema](https://stencila.org/ListOrder.schema.json)
- Python type [`ListOrder`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`ListOrder`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/list_order.rs)
- TypeScript type [`ListOrder`](https://github.com/stencila/stencila/blob/main/ts/src/types/ListOrder.ts)

***

This documentation was generated from [`ListOrder.yaml`](https://github.com/stencila/stencila/blob/main/schema/ListOrder.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
