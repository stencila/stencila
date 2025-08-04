---
title: List
description: A list of items.
config:
  publish:
    ghost:
      type: post
      slug: list
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

This is an implementation, and renaming, of schema.org [`ItemList`](https://schema.org/ItemList).
Renaming was done as `List` was considered a more developer friendly alternative. Similarly,
schema.org properties `itemListElement` and `itemListOrder` were renamed to `items` and `order`.
Note that, as with every other such renaming in Stencila Schema, a mapping between names is
defined and it is trivial to save Stencila Schema documents using the schema.org vocabulary if so desired.


# Properties

The `List` type has these properties:

| Name         | Description                                                 | Type                                                                                   | Inherited from                                                     | `JSON-LD @id`                                                  | Aliases  |
| ------------ | ----------------------------------------------------------- | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------------------------- | -------- |
| `id`         | The identifier for this item.                               | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                     | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                           | -        |
| `items`      | The items in the list.                                      | [`ListItem`](https://stencila.ghost.io/docs/reference/schema/list-item)*               | -                                                                  | [`schema:itemListElement`](https://schema.org/itemListElement) | `item`   |
| `order`      | The ordering of the list.                                   | [`ListOrder`](https://stencila.ghost.io/docs/reference/schema/list-order)              | -                                                                  | [`schema:itemListOrder`](https://schema.org/itemListOrder)     | -        |
| `authors`    | The authors of the list.                                    | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                    | -                                                                  | [`schema:author`](https://schema.org/author)                   | `author` |
| `provenance` | A summary of the provenance of the content within the list. | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)* | -                                                                  | `stencila:provenance`                                          | -        |

# Related

The `List` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `List` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                                                                                            | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | -------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                                                                                    |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              | Encoded using special function                                                                     |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       | 🔷 Low loss   |              | Encoded as [`<list>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/list.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | 🔷 Low loss   | 🔷 Low loss   | Encoded using implemented function                                                                 |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                    |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                                                                                    |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |                                                                                                    |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                                                                                    |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |                                                                                                    |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                                                                                    |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                                                                                    |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                                                                                    |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                                                                                    |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |                                                                                                    |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                                                                                    |

# Bindings

The `List` type is represented in:

- [JSON-LD](https://stencila.org/List.jsonld)
- [JSON Schema](https://stencila.org/List.schema.json)
- Python class [`List`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/list.py)
- Rust struct [`List`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/list.rs)
- TypeScript class [`List`](https://github.com/stencila/stencila/blob/main/ts/src/types/List.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `List` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property | Complexity | Description                                                | Strategy                                                             |
| -------- | ---------- | ---------------------------------------------------------- | -------------------------------------------------------------------- |
| `items`  | Min+       | Generate a single, arbitrary, list item.                   | `vec(ListItem::arbitrary(), size_range(1..=1))`                      |
|          | Low+       | Generate up to two, arbitrary, list items.                 | `vec(ListItem::arbitrary(), size_range(1..=2))`                      |
|          | High+      | Generate up to four, arbitrary, list items.                | `vec(ListItem::arbitrary(), size_range(1..=4))`                      |
|          | Max        | Generate up to eight, arbitrary, list items.               | `vec(ListItem::arbitrary(), size_range(1..=8))`                      |
| `order`  | Min+       | Always generate an unordered list.                         | `ListOrder::Unordered`                                               |
|          | Low+       | Randomly generate either an unordered, or ascending, list. | `prop_oneof![Just(ListOrder::Unordered),Just(ListOrder::Ascending)]` |
|          | High+      | Generate an arbitrary list ordering.                       | `ListOrder::arbitrary()`                                             |

# Source

This documentation was generated from [`List.yaml`](https://github.com/stencila/stencila/blob/main/schema/List.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
