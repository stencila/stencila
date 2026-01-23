---
title: Datatable Column Hint
description: A hint to the type and values in a `DatatableColumn`.
---

# Properties

The `DatatableColumnHint` type has these properties:

| Name       | Description                                | Type                          | Inherited from          |
| ---------- | ------------------------------------------ | ----------------------------- | ----------------------- |
| `id`       | The identifier for this item.              | [`String`](./string.md)       | [`Entity`](./entity.md) |
| `name`     | The name of the column.                    | [`String`](./string.md)       | -                       |
| `itemType` | The type of items in the column.           | [`String`](./string.md)       | -                       |
| `minimum`  | The minimum value in the column.           | [`Primitive`](./primitive.md) | -                       |
| `maximum`  | The maximum value in the column.           | [`Primitive`](./primitive.md) | -                       |
| `nulls`    | The number of `Null` values in the column. | [`Integer`](./integer.md)     | -                       |

# Related

The `DatatableColumnHint` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `DatatableColumnHint` type is represented in:

- [JSON-LD](https://stencila.org/DatatableColumnHint.jsonld)
- [JSON Schema](https://stencila.org/DatatableColumnHint.schema.json)
- Python class [`DatatableColumnHint`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`DatatableColumnHint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/datatable_column_hint.rs)
- TypeScript class [`DatatableColumnHint`](https://github.com/stencila/stencila/blob/main/ts/src/types/DatatableColumnHint.ts)

***

This documentation was generated from [`DatatableColumnHint.yaml`](https://github.com/stencila/stencila/blob/main/schema/DatatableColumnHint.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
