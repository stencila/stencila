---
title: Table Cell
description: A cell within a `Table`.
config:
  publish:
    ghost:
      type: post
      slug: table-cell
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Works
---

# Properties

The `TableCell` type has these properties:

| Name                           | Description                                                                      | Type                                                                                          | Inherited from                                                     | `JSON-LD @id`                            | Aliases                                                            |
| ------------------------------ | -------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ---------------------------------------- | ------------------------------------------------------------------ |
| `id`                           | The identifier for this item.                                                    | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                            | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)     | -                                                                  |
| `cellType`                     | The type of cell.                                                                | [`TableCellType`](https://stencila.ghost.io/docs/reference/schema/table-cell-type)            | -                                                                  | `stencila:cellType`                      | `cell-type`, `cell_type`                                           |
| `name`                         | The name of the cell.                                                            | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                            | -                                                                  | [`schema:name`](https://schema.org/name) | -                                                                  |
| `columnSpan`                   | How many columns the cell extends.                                               | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)                          | -                                                                  | `stencila:colspan`                       | `column-span`, `column_span`                                       |
| `rowSpan`                      | How many columns the cell extends.                                               | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)                          | -                                                                  | `stencila:rowspan`                       | `row-span`, `row_span`                                             |
| `horizontalAlignment`          | The horizontal alignment of the content of a table cell.                         | [`HorizontalAlignment`](https://stencila.ghost.io/docs/reference/schema/horizontal-alignment) | -                                                                  | `stencila:horizontalAlignment`           | `horizontal-alignment`, `horizontal_alignment`                     |
| `horizontalAlignmentCharacter` | The character to be used in horizontal alignment of the content of a table cell. | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                            | -                                                                  | `stencila:horizontalAlignmentCharacter`  | `horizontal-alignment-character`, `horizontal_alignment_character` |
| `verticalAlignment`            | The vertical alignment of the content of a table cell.                           | [`VerticalAlignment`](https://stencila.ghost.io/docs/reference/schema/vertical-alignment)     | -                                                                  | `stencila:verticalAlignment`             | `vertical-alignment`, `vertical_alignment`                         |
| `content`                      | Contents of the table cell.                                                      | [`Block`](https://stencila.ghost.io/docs/reference/schema/block)*                             | -                                                                  | `stencila:content`                       | -                                                                  |

# Related

The `TableCell` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `TableCell` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                                                                           | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | --------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                                                                   |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              | Encoded as [`<td>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td) |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |              |              | Encoded using special function                                                    |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                                                                   |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                                                                   |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |                                                                                   |
| [OpenDocument Text](https://stencila.ghost.io/docs/reference/formats/odt)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                                                                   |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                                                                   |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                                                                   |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                                                                   |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                                                                   |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |                                                                                   |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                                                                   |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                                                                   |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |                                                                                   |
| [CSV](https://stencila.ghost.io/docs/reference/formats/csv)                         |              |              |                                                                                   |
| [TSV](https://stencila.ghost.io/docs/reference/formats/tsv)                         |              |              |                                                                                   |
| [Microsoft Excel XLSX](https://stencila.ghost.io/docs/reference/formats/xlsx)       |              |              |                                                                                   |
| [Microsoft Excel XLS](https://stencila.ghost.io/docs/reference/formats/xls)         |              |              |                                                                                   |
| [OpenDocument Spreadsheet](https://stencila.ghost.io/docs/reference/formats/ods)    |              |              |                                                                                   |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                                                                   |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                                                                   |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                                                                   |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                                                                   |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |                                                                                   |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                                                                   |

# Bindings

The `TableCell` type is represented in:

- [JSON-LD](https://stencila.org/TableCell.jsonld)
- [JSON Schema](https://stencila.org/TableCell.schema.json)
- Python class [`TableCell`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/table_cell.py)
- Rust struct [`TableCell`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table_cell.rs)
- TypeScript class [`TableCell`](https://github.com/stencila/stencila/blob/main/ts/src/types/TableCell.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `TableCell` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                             | Strategy                |
| --------- | ---------- | --------------------------------------- | ----------------------- |
| `content` | Min+       | Generate a single, arbitrary, paragraph | `table_cell_content(1)` |

# Source

This documentation was generated from [`TableCell.yaml`](https://github.com/stencila/stencila/blob/main/schema/TableCell.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
