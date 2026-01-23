---
title: Table Cell
description: A cell within a `Table`.
---

# Properties

The `TableCell` type has these properties:

| Name                           | Description                                                                      | Type                                               | Inherited from          |
| ------------------------------ | -------------------------------------------------------------------------------- | -------------------------------------------------- | ----------------------- |
| `id`                           | The identifier for this item.                                                    | [`String`](./string.md)                            | [`Entity`](./entity.md) |
| `cellType`                     | The type of cell.                                                                | [`TableCellType`](./table-cell-type.md)            | -                       |
| `name`                         | The name of the cell.                                                            | [`String`](./string.md)                            | -                       |
| `columnSpan`                   | How many columns the cell extends.                                               | [`Integer`](./integer.md)                          | -                       |
| `rowSpan`                      | How many columns the cell extends.                                               | [`Integer`](./integer.md)                          | -                       |
| `horizontalAlignment`          | The horizontal alignment of the content of a table cell.                         | [`HorizontalAlignment`](./horizontal-alignment.md) | -                       |
| `horizontalAlignmentCharacter` | The character to be used in horizontal alignment of the content of a table cell. | [`String`](./string.md)                            | -                       |
| `verticalAlignment`            | The vertical alignment of the content of a table cell.                           | [`VerticalAlignment`](./vertical-alignment.md)     | -                       |
| `content`                      | Contents of the table cell.                                                      | [`Block`](./block.md)*                             | -                       |

# Related

The `TableCell` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `TableCell` type is represented in:

- [JSON-LD](https://stencila.org/TableCell.jsonld)
- [JSON Schema](https://stencila.org/TableCell.schema.json)
- Python class [`TableCell`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`TableCell`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table_cell.rs)
- TypeScript class [`TableCell`](https://github.com/stencila/stencila/blob/main/ts/src/types/TableCell.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `TableCell` type are generated using the following strategies.

::: table

| Property  | Complexity | Description                             | Strategy                |
| --------- | ---------- | --------------------------------------- | ----------------------- |
| `content` | Min+       | Generate a single, arbitrary, paragraph | `table_cell_content(1)` |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`TableCell.yaml`](https://github.com/stencila/stencila/blob/main/schema/TableCell.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
