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

| Name         | Description                                                 | Type                                        | Inherited from          | `JSON-LD @id`                                                  | Aliases  |
| ------------ | ----------------------------------------------------------- | ------------------------------------------- | ----------------------- | -------------------------------------------------------------- | -------- |
| `id`         | The identifier for this item.                               | [`String`](./string.md)                     | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)                           | -        |
| `items`      | The items in the list.                                      | [`ListItem`](./list-item.md)*               | -                       | [`schema:itemListElement`](https://schema.org/itemListElement) | `item`   |
| `order`      | The ordering of the list.                                   | [`ListOrder`](./list-order.md)              | -                       | [`schema:itemListOrder`](https://schema.org/itemListOrder)     | -        |
| `authors`    | The authors of the list.                                    | [`Author`](./author.md)*                    | -                       | [`schema:author`](https://schema.org/author)                   | `author` |
| `provenance` | A summary of the provenance of the content within the list. | [`ProvenanceCount`](./provenance-count.md)* | -                       | `stencila:provenance`                                          | -        |

# Related

The `List` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `List` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                            | Notes |
| ------------------------------------------------ | ------------ | ------------ | -------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                    |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              | Encoded using special function                                                                     |
| [JATS](../formats/jats.md)                       | 🔷 Low loss   |              | Encoded as [`<list>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/list.html) |
| [Markdown](../formats/md.md)                     | 🔷 Low loss   | 🔷 Low loss   | Encoded using implemented function                                                                 |
| [Stencila Markdown](../formats/smd.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Quarto Markdown](../formats/qmd.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [MyST Markdown](../formats/myst.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [LLM Markdown](../formats/llmd.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                    |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                                    |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                                    |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                                    |
| [CSV](../formats/csv.md)                         |              |              |                                                                                                    |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                                    |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                                    |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                                    |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                                    |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                                    |
| [Directory](../formats/directory.md)             |              |              |                                                                                                    |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                                    |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                                    |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                                    |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                                    |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                                    |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                                    |

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
