---
title: Table Row
description: A row within a Table.
config:
  publish:
    ghost:
      type: post
      slug: table-row
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Works
---

# Properties

The `TableRow` type has these properties:

| Name      | Description                   | Type                                                                             | Inherited from                                                     | `JSON-LD @id`                        | Aliases                |
| --------- | ----------------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------ | ---------------------- |
| `id`      | The identifier for this item. | [`String`](https://stencila.ghost.io/docs/reference/schema/string)               | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -                      |
| `cells`   | An array of cells in the row. | [`TableCell`](https://stencila.ghost.io/docs/reference/schema/table-cell)*       | -                                                                  | `stencila:cells`                     | `cell`                 |
| `rowType` | The type of row.              | [`TableRowType`](https://stencila.ghost.io/docs/reference/schema/table-row-type) | -                                                                  | `stencila:rowType`                   | `row-type`, `row_type` |

# Related

The `TableRow` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `TableRow` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                       | Encoding     | Decoding   | Support                                                                           | Notes |
| ---------------------------------------------------------------------------- | ------------ | ---------- | --------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)        | 🟢 No loss    |            |                                                                                   |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                | 🔷 Low loss   |            | Encoded as [`<tr>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr) |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                |              |            | Encoded using special function                                                    |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)              | 🔷 Low loss   | 🔷 Low loss |                                                                                   |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)    | 🔷 Low loss   | 🔷 Low loss |                                                                                   |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)      | 🔷 Low loss   | 🔷 Low loss |                                                                                   |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)       | 🔷 Low loss   | 🔷 Low loss |                                                                                   |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)        | 🔷 Low loss   | 🔷 Low loss |                                                                                   |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)              | 🔷 Low loss   | 🔷 Low loss |                                                                                   |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                  | 🔷 Low loss   |            |                                                                                   |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)          | ⚠️ High loss |            |                                                                                   |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)              | 🔷 Low loss   | 🔷 Low loss |                                                                                   |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx) | 🔷 Low loss   | 🔷 Low loss |                                                                                   |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)     | 🔷 Low loss   | 🔷 Low loss |                                                                                   |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                  | 🔷 Low loss   | 🔷 Low loss |                                                                                   |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                | 🟢 No loss    | 🟢 No loss  |                                                                                   |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)        | 🟢 No loss    | 🟢 No loss  |                                                                                   |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)              | 🟢 No loss    | 🟢 No loss  |                                                                                   |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)           | 🟢 No loss    | 🟢 No loss  |                                                                                   |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                | 🟢 No loss    | 🟢 No loss  |                                                                                   |
| [CBOR+Zstandard](https://stencila.ghost.io/docs/reference/formats/cbor.zstd) | 🟢 No loss    | 🟢 No loss  |                                                                                   |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                | 🟢 No loss    | 🟢 No loss  |                                                                                   |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)     | 🔷 Low loss   | 🔷 Low loss |                                                                                   |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)       | 🔷 Low loss   | 🔷 Low loss |                                                                                   |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)        | 🔷 Low loss   | 🔷 Low loss |                                                                                   |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)      |              |            |                                                                                   |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)  |              |            |                                                                                   |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)              | 🔷 Low loss   |            |                                                                                   |

# Bindings

The `TableRow` type is represented in:

- [JSON-LD](https://stencila.org/TableRow.jsonld)
- [JSON Schema](https://stencila.org/TableRow.schema.json)
- Python class [`TableRow`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/table_row.py)
- Rust struct [`TableRow`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table_row.rs)
- TypeScript class [`TableRow`](https://github.com/stencila/stencila/blob/main/ts/src/types/TableRow.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `TableRow` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property | Complexity | Description                                   | Strategy                                         |
| -------- | ---------- | --------------------------------------------- | ------------------------------------------------ |
| `cells`  | Min+       | Generate a single, arbitrary, table cell.     | `vec(TableCell::arbitrary(), size_range(1..=1))` |
|          | Low+       | Generate two, arbitrary, table cells.         | `vec(TableCell::arbitrary(), size_range(2..=2))` |
|          | High+      | Generate four, arbitrary, table cells.        | `vec(TableCell::arbitrary(), size_range(4..=4))` |
|          | Max        | Generate up to eight, arbitrary, table cells. | `vec(TableCell::arbitrary(), size_range(1..=8))` |

# Source

This documentation was generated from [`TableRow.yaml`](https://github.com/stencila/stencila/blob/main/schema/TableRow.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
