---
title: Datatable Column
description: A column of data within a `Datatable`.
---

# Properties

The `DatatableColumn` type has these properties:

| Name        | Description                                          | Type                                     | Inherited from          |
| ----------- | ---------------------------------------------------- | ---------------------------------------- | ----------------------- |
| `id`        | The identifier for this item.                        | [`String`](./string.md)                  | [`Entity`](./entity.md) |
| `name`      | The name of the column.                              | [`String`](./string.md)                  | -                       |
| `values`    | The data values of the column.                       | [`Primitive`](./primitive.md)*           | -                       |
| `validator` | The validator to use to validate data in the column. | [`ArrayValidator`](./array-validator.md) | -                       |

# Related

The `DatatableColumn` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `DatatableColumn` type is represented in:

- [JSON-LD](https://stencila.org/DatatableColumn.jsonld)
- [JSON Schema](https://stencila.org/DatatableColumn.schema.json)
- Python class [`DatatableColumn`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`DatatableColumn`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/datatable_column.rs)
- TypeScript class [`DatatableColumn`](https://github.com/stencila/stencila/blob/main/ts/src/types/DatatableColumn.ts)

***

This documentation was generated from [`DatatableColumn.yaml`](https://github.com/stencila/stencila/blob/main/schema/DatatableColumn.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
