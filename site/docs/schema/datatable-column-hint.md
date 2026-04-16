---
title: Datatable Column Hint
description: A concise summary of the properties of a `DatatableColumn`.
---

This is a type used in Stencila Schema for providing a concise summary of the
properties of a [`DatatableColumn`](./datatable-column.md).

It exists to support both human and machine understanding of tabular columns,
including schema inference, editing assistance, and code generation
workflows such as selecting visual encodings based on column types. Rather
than making observations into hard constraints, it summarizes inferred type,
value range, and other characteristics of the column.

Key properties describe inferred value kinds, ranges, and representative
examples.


# Properties

The `DatatableColumnHint` type has these properties:

| Name       | Description                                | Type                          | Inherited from          |
| ---------- | ------------------------------------------ | ----------------------------- | ----------------------- |
| `name`     | The name of the column.                    | [`String`](./string.md)       | -                       |
| `itemType` | The type of items in the column.           | [`String`](./string.md)       | -                       |
| `minimum`  | The minimum value in the column.           | [`Primitive`](./primitive.md) | -                       |
| `maximum`  | The maximum value in the column.           | [`Primitive`](./primitive.md) | -                       |
| `nulls`    | The number of `Null` values in the column. | [`Integer`](./integer.md)     | -                       |
| `id`       | The identifier for this item.              | [`String`](./string.md)       | [`Entity`](./entity.md) |

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
