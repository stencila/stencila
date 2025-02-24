---
title: Table Cell Type
description: Indicates whether the cell is a header or data.
config:
  publish:
    ghost:
      type: post
      slug: table-cell-type
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Works
---

When `HeaderCell`, the cell is similar to the HTML [`<th>` element](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th)).
When `DataCell`, the cell is similar to the HTML [`<td>` element](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td)).


# Members

The `TableCellType` type has these members:

- `DataCell`
- `HeaderCell`

# Bindings

The `TableCellType` type is represented in:

- [JSON-LD](https://stencila.org/TableCellType.jsonld)
- [JSON Schema](https://stencila.org/TableCellType.schema.json)
- Python type [`TableCellType`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/table_cell_type.py)
- Rust type [`TableCellType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table_cell_type.rs)
- TypeScript type [`TableCellType`](https://github.com/stencila/stencila/blob/main/ts/src/types/TableCellType.ts)

# Source

This documentation was generated from [`TableCellType.yaml`](https://github.com/stencila/stencila/blob/main/schema/TableCellType.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
