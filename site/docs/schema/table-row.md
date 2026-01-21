---
title: Table Row
description: A row within a Table.
---

# Properties

The `TableRow` type has these properties:

| Name      | Description                   | Type                                  | Inherited from          |
| --------- | ----------------------------- | ------------------------------------- | ----------------------- |
| `id`      | The identifier for this item. | [`String`](./string.md)               | [`Entity`](./entity.md) |
| `cells`   | An array of cells in the row. | [`TableCell`](./table-cell.md)*       | -                       |
| `rowType` | The type of row.              | [`TableRowType`](./table-row-type.md) | -                       |

# Related

The `TableRow` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `TableRow` type is represented in:

- [JSON-LD](https://stencila.org/TableRow.jsonld)
- [JSON Schema](https://stencila.org/TableRow.schema.json)
- Python class [`TableRow`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/table_row.py)
- Rust struct [`TableRow`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table_row.rs)
- TypeScript class [`TableRow`](https://github.com/stencila/stencila/blob/main/ts/src/types/TableRow.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `TableRow` type are generated using the following strategies.

::: table

| Property | Complexity | Description                                   | Strategy                                         |
| -------- | ---------- | --------------------------------------------- | ------------------------------------------------ |
| `cells`  | Min+       | Generate a single, arbitrary, table cell.     | `vec(TableCell::arbitrary(), size_range(1..=1))` |
|          | Low+       | Generate two, arbitrary, table cells.         | `vec(TableCell::arbitrary(), size_range(2..=2))` |
|          | High+      | Generate four, arbitrary, table cells.        | `vec(TableCell::arbitrary(), size_range(4..=4))` |
|          | Max        | Generate up to eight, arbitrary, table cells. | `vec(TableCell::arbitrary(), size_range(1..=8))` |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`TableRow.yaml`](https://github.com/stencila/stencila/blob/main/schema/TableRow.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
