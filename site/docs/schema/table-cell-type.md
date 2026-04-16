---
title: Table Cell Type
description: The structural role of a table cell.
---

This is an enumeration used in Stencila Schema for table cell roles.

It exists so table cells can be identified as data cells or header cells using
a stable vocabulary that maps cleanly to HTML, JATS, and other tabular
formats.

See [`TableCell.cellType`](./table-cell.md#celltype) for the property that
uses this enumeration.


# Analogues

The following external types, elements, or nodes are similar to a `TableCellType`:

- [HTML td/th distinction](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td): Close analogue because HTML distinguishes data and header cells structurally with separate elements rather than an enumeration.

# Members

The `TableCellType` type has these members:

| Member       | Description |
| ------------ | ----------- |
| `DataCell`   | -           |
| `HeaderCell` | -           |

# Bindings

The `TableCellType` type is represented in:

- [JSON-LD](https://stencila.org/TableCellType.jsonld)
- [JSON Schema](https://stencila.org/TableCellType.schema.json)
- Python type [`TableCellType`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`TableCellType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table_cell_type.rs)
- TypeScript type [`TableCellType`](https://github.com/stencila/stencila/blob/main/ts/src/types/TableCellType.ts)

***

This documentation was generated from [`TableCellType.yaml`](https://github.com/stencila/stencila/blob/main/schema/TableCellType.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
