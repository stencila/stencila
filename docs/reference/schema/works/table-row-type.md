# Table Row Type

**Indicates whether the row is in the header, body or footer of the table.**

**`@id`**: `stencila:TableRowType`

## Members

The `TableRowType` type has these members:

- `HeaderRow`
- `BodyRow`
- `FooterRow`

## Bindings

The `TableRowType` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/TableRowType.jsonld)
- [JSON Schema](https://stencila.dev/TableRowType.schema.json)
- Python type [`TableRowType`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/table_row_type.py)
- Rust type [`TableRowType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table_row_type.rs)
- TypeScript type [`TableRowType`](https://github.com/stencila/stencila/blob/main/typescript/src/types/TableRowType.ts)

## Testing

During property-based (a.k.a generative) testing, the variants of the `TableRowType` type are generated using the following strategies for each complexity level (see the [`proptest` book](https://proptest-rs.github.io/proptest/) for an explanation of the Rust strategy expressions). Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

|         |            |             |          |
| ------- | ---------- | ----------- | -------- |
| Variant | Complexity | Description | Strategy |

## Source

This documentation was generated from [`TableRowType.yaml`](https://github.com/stencila/stencila/blob/main/schema/TableRowType.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).