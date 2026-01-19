---
title: Table Cell
description: A cell within a `Table`.
---

# Properties

The `TableCell` type has these properties:

| Name                           | Description                                                                      | Type                                               | Inherited from          | `JSON-LD @id`                            | Aliases                                                            |
| ------------------------------ | -------------------------------------------------------------------------------- | -------------------------------------------------- | ----------------------- | ---------------------------------------- | ------------------------------------------------------------------ |
| `id`                           | The identifier for this item.                                                    | [`String`](./string.md)                            | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)     | -                                                                  |
| `cellType`                     | The type of cell.                                                                | [`TableCellType`](./table-cell-type.md)            | -                       | `stencila:cellType`                      | `cell-type`, `cell_type`                                           |
| `name`                         | The name of the cell.                                                            | [`String`](./string.md)                            | -                       | [`schema:name`](https://schema.org/name) | -                                                                  |
| `columnSpan`                   | How many columns the cell extends.                                               | [`Integer`](./integer.md)                          | -                       | `stencila:columnSpan`                    | `column-span`, `column_span`                                       |
| `rowSpan`                      | How many columns the cell extends.                                               | [`Integer`](./integer.md)                          | -                       | `stencila:rowSpan`                       | `row-span`, `row_span`                                             |
| `horizontalAlignment`          | The horizontal alignment of the content of a table cell.                         | [`HorizontalAlignment`](./horizontal-alignment.md) | -                       | `stencila:horizontalAlignment`           | `horizontal-alignment`, `horizontal_alignment`                     |
| `horizontalAlignmentCharacter` | The character to be used in horizontal alignment of the content of a table cell. | [`String`](./string.md)                            | -                       | `stencila:horizontalAlignmentCharacter`  | `horizontal-alignment-character`, `horizontal_alignment_character` |
| `verticalAlignment`            | The vertical alignment of the content of a table cell.                           | [`VerticalAlignment`](./vertical-alignment.md)     | -                       | `stencila:verticalAlignment`             | `vertical-alignment`, `vertical_alignment`                         |
| `content`                      | Contents of the table cell.                                                      | [`Block`](./block.md)*                             | -                       | `stencila:content`                       | -                                                                  |

# Related

The `TableCell` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `TableCell` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                           | Notes |
| ------------------------------------------------ | ------------ | ------------ | --------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                   |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              | Encoded as [`<td>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td) |
| [JATS](../formats/jats.md)                       |              |              | Encoded using special function                                                    |
| [Markdown](../formats/md.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [Stencila Markdown](../formats/smd.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [Quarto Markdown](../formats/qmd.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [MyST Markdown](../formats/myst.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [LLM Markdown](../formats/llmd.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                   |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                   |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                   |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                   |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                   |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                   |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                   |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                   |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                   |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                   |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                   |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                   |
| [CSV](../formats/csv.md)                         |              |              |                                                                                   |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                   |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                   |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                   |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                   |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                   |
| [Directory](../formats/directory.md)             |              |              |                                                                                   |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                   |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                   |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                   |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                   |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                   |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                   |

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
