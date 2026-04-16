---
title: Table Row
description: A row within a Table.
---

This is a type used in Stencila Schema for rows within a [`Table`](./table.md).

It exists to represent row structure explicitly so header, body, and footer
semantics can be preserved across transformations and renderers.

Key properties include the row `cells` and `rowType`.


# Analogues

The following external types, elements, or nodes are similar to a `TableRow`:

- HTML [`<tr>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr)
- Pandoc [`Table`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Table): Only an approximate analogue because Pandoc rows are nested structures inside tables rather than standalone row nodes.

# Properties

The `TableRow` type has these properties:

| Name      | Description                   | Type                                  | Inherited from          |
| --------- | ----------------------------- | ------------------------------------- | ----------------------- |
| `cells`   | An array of cells in the row. | [`TableCell`](./table-cell.md)*       | -                       |
| `rowType` | The type of row.              | [`TableRowType`](./table-row-type.md) | -                       |
| `id`      | The identifier for this item. | [`String`](./string.md)               | [`Entity`](./entity.md) |

# Related

The `TableRow` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `TableRow` type is represented in:

- [JSON-LD](https://stencila.org/TableRow.jsonld)
- [JSON Schema](https://stencila.org/TableRow.schema.json)
- Python class [`TableRow`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
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

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`TableRow.yaml`](https://github.com/stencila/stencila/blob/main/schema/TableRow.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
