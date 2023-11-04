# Table Cell Type

**Indicates whether the cell is a header or data.**

When `Header`, the cell is similar to the HTML [`<th>` element](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th)).
When `Data`, the cell is similar to the HTML [`<td>` element](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td)).


**`@id`**: `stencila:TableCellType`

## Members

The `TableCellType` type has these members:

- `DataCell`
- `HeaderCell`

## Bindings

The `TableCellType` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/TableCellType.jsonld)
- [JSON Schema](https://stencila.dev/TableCellType.schema.json)
- Python type [`TableCellType`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/table_cell_type.py)
- Rust type [`TableCellType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table_cell_type.rs)
- TypeScript type [`TableCellType`](https://github.com/stencila/stencila/blob/main/typescript/src/types/TableCellType.ts)

## Testing

During property-based (a.k.a generative) testing, the variants of the `TableCellType` type are generated using the following strategies for each complexity level (see the [`proptest` book](https://proptest-rs.github.io/proptest/) for an explanation of the Rust strategy expressions). Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

|         |            |             |          |
| ------- | ---------- | ----------- | -------- |
| Variant | Complexity | Description | Strategy |

## Source

This documentation was generated from [`TableCellType.yaml`](https://github.com/stencila/stencila/blob/main/schema/TableCellType.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).