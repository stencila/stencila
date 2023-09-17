---
title:
- type: Text
  value: TableRow
---

# Table Row

**A row within a Table.**

**`@id`**: `stencila:TableRow`

## Properties

The `TableRow` type has these properties:

| Name    | `@id`                                | Type                                                                              | Description                   | Inherited from                                                           |
| ------- | ------------------------------------ | --------------------------------------------------------------------------------- | ----------------------------- | ------------------------------------------------------------------------ |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                | The identifier for this item  | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)      |
| cells   | `stencila:cells`                     | [`TableCell`](https://stencila.dev/docs/reference/schema/works/table-cell)*       | An array of cells in the row. | [`TableRow`](https://stencila.dev/docs/reference/schema/works/table-row) |
| rowType | `stencila:rowType`                   | [`TableRowType`](https://stencila.dev/docs/reference/schema/works/table-row-type) | The type of row.              | [`TableRow`](https://stencila.dev/docs/reference/schema/works/table-row) |

## Related

The `TableRow` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `TableRow` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                 |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | --------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag `<tr>` |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🔷 Low loss     |              | 🚧 Under development    |                       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                       |

## Bindings

The `TableRow` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/TableRow.jsonld)
- [JSON Schema](https://stencila.dev/TableRow.schema.json)
- Python class [`TableRow`](https://github.com/stencila/stencila/blob/main/python/stencila/types/table_row.py)
- Rust struct [`TableRow`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table_row.rs)
- TypeScript class [`TableRow`](https://github.com/stencila/stencila/blob/main/typescript/src/types/TableRow.ts)

## Source

This documentation was generated from [`TableRow.yaml`](https://github.com/stencila/stencila/blob/main/schema/TableRow.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).