---
title:
- type: Text
  value: TableCell
---

# Table Cell

**A cell within a `Table`.**

**`@id`**: `stencila:TableCell`

## Properties

The `TableCell` type has these properties:

| Name       | `@id`                                    | Type                                                                                                                                      | Description                         | Inherited from                                                             |
| ---------- | ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------- | -------------------------------------------------------------------------- |
| id         | [`schema:id`](https://schema.org/id)     | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                        | The identifier for this item        | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)        |
| cellType   | `stencila:cellType`                      | [`TableCellType`](https://stencila.dev/docs/reference/schema/works/table-cell-type)                                                       | The type of cell.                   | [`TableCell`](https://stencila.dev/docs/reference/schema/works/table-cell) |
| name       | [`schema:name`](https://schema.org/name) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                        | The name of the cell.               | [`TableCell`](https://stencila.dev/docs/reference/schema/works/table-cell) |
| columnSpan | `stencila:colspan`                       | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer)                                                                      | How many columns the cell extends.  | [`TableCell`](https://stencila.dev/docs/reference/schema/works/table-cell) |
| rowSpan    | `stencila:rowspan`                       | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer)                                                                      | How many columns the cell extends.  | [`TableCell`](https://stencila.dev/docs/reference/schema/works/table-cell) |
| content    | `stencila:content`                       | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)* \| [`Inline`](https://stencila.dev/docs/reference/schema/prose/inline)* | Contents of the table cell.         | [`TableCell`](https://stencila.dev/docs/reference/schema/works/table-cell) |

## Related

The `TableCell` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `TableCell` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                                                                 |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ------------------------------------------------------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<td>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td) |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |                                                                                       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🔷 Low loss     |              | 🚧 Under development    |                                                                                       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                                                                       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                                                                       |

## Bindings

The `TableCell` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/TableCell.jsonld)
- [JSON Schema](https://stencila.dev/TableCell.schema.json)
- Python class [`TableCell`](https://github.com/stencila/stencila/blob/main/python/stencila/types/table_cell.py)
- Rust struct [`TableCell`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/table_cell.rs)
- TypeScript class [`TableCell`](https://github.com/stencila/stencila/blob/main/typescript/src/types/TableCell.ts)

## Source

This documentation was generated from [`TableCell.yaml`](https://github.com/stencila/stencila/blob/main/schema/TableCell.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).