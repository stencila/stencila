---
title:
- type: Text
  value: List
---

# List

**A list of items.**

This is an implementation, and renaming, of schema.org [`ItemList`](https://schema.org/ItemList).
Renaming was done as `List` was considered a more developer friendly alternative. Similarly,
schema.org properties `itemListElement` and `itemListOrder` were renamed to `items` and `order`.
Note that, as with every other such renaming in Stencila Schema, a mapping between names is
defined and it is trivial to save Stencila Schema documents using the schema.org vocabulary if so desired.


**`@id`**: [`schema:ItemList`](https://schema.org/ItemList)

## Properties

The `List` type has these properties:

| Name  | `@id`                                                          | Type                                                                       | Description                  | Inherited from                                                      |
| ----- | -------------------------------------------------------------- | -------------------------------------------------------------------------- | ---------------------------- | ------------------------------------------------------------------- |
| id    | [`schema:id`](https://schema.org/id)                           | [`String`](https://stencila.dev/docs/reference/schema/data/string)         | The identifier for this item | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| items | [`schema:itemListElement`](https://schema.org/itemListElement) | [`ListItem`](https://stencila.dev/docs/reference/schema/prose/list-item)*  | The items in the list.       | [`List`](https://stencila.dev/docs/reference/schema/prose/list)     |
| order | [`schema:itemListOrder`](https://schema.org/itemListOrder)     | [`ListOrder`](https://stencila.dev/docs/reference/schema/prose/list-order) | The ordering of the list.    | [`List`](https://stencila.dev/docs/reference/schema/prose/list)     |

## Related

The `List` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `List` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                                                                             |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ------------------------------------------------------------------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded using special function                                                                    |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<list>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/list) |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🔷 Low loss     |              | 🚧 Under development    | Encoded using special function                                                                    |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                                                                                   |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                   |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                   |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                   |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                                                                                   |

## Bindings

The `List` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/List.jsonld)
- [JSON Schema](https://stencila.dev/List.schema.json)
- Python class [`List`](https://github.com/stencila/stencila/blob/main/python/stencila/types/list.py)
- Rust struct [`List`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/list.rs)
- TypeScript class [`List`](https://github.com/stencila/stencila/blob/main/typescript/src/types/List.ts)

## Source

This documentation was generated from [`List.yaml`](https://github.com/stencila/stencila/blob/main/schema/List.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).