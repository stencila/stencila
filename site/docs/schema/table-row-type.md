---
title: Table Row Type
description: The structural role of a table row.
---

This is an enumeration used in Stencila Schema for the structural role of a table
row.

It exists so rows can be identified as header, body, or footer rows using a
stable vocabulary that maps cleanly across formats.

See [`TableRow.rowType`](./table-row.md#rowtype) for the property that uses
this enumeration.


# Analogues

The following external types, elements, or nodes are similar to a `TableRowType`:

- [HTML table section semantics](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table): Approximate analogue because HTML distinguishes header, body, and footer by row groups such as `<thead>`, `<tbody>`, and `<tfoot>` rather than a row-type enumeration.

# Members

The `TableRowType` type has these members:

| Member      | Description |
| ----------- | ----------- |
| `HeaderRow` | -           |
| `BodyRow`   | -           |
| `FooterRow` | -           |

# Bindings

The `TableRowType` type is represented in:

- [JSON-LD](https://stencila.org/TableRowType.jsonld)
- [JSON Schema](https://stencila.org/TableRowType.schema.json)
- Python type [`TableRowType`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`TableRowType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table_row_type.rs)
- TypeScript type [`TableRowType`](https://github.com/stencila/stencila/blob/main/ts/src/types/TableRowType.ts)

***

This documentation was generated from [`TableRowType.yaml`](https://github.com/stencila/stencila/blob/main/schema/TableRowType.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
