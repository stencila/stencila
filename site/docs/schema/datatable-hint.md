---
title: Datatable Hint
description: A concise summary of the structure of a table of data.
---

This is a type used in Stencila Schema for providing a concise summary of the
structure of a [`Datatable`](./datatable.md).

It exists to support both human and machine understanding of tabular data,
including schema inference, import workflows, user interfaces, and code
generation workflows such as choosing suitable visualizations for large
datasets. Unlike validators, it summarizes observed or inferred structure
rather than enforcing it.

Key properties include per-column hints and dataset-level observations.


# Properties

The `DatatableHint` type has these properties:

| Name      | Description                     | Type                                                 | Inherited from          |
| --------- | ------------------------------- | ---------------------------------------------------- | ----------------------- |
| `rows`    | The number of rows of data.     | [`Integer`](./integer.md)                            | -                       |
| `columns` | A hint for each column of data. | [`DatatableColumnHint`](./datatable-column-hint.md)* | -                       |
| `id`      | The identifier for this item.   | [`String`](./string.md)                              | [`Entity`](./entity.md) |

# Related

The `DatatableHint` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `DatatableHint` type is represented in:

- [JSON-LD](https://stencila.org/DatatableHint.jsonld)
- [JSON Schema](https://stencila.org/DatatableHint.schema.json)
- Python class [`DatatableHint`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`DatatableHint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/datatable_hint.rs)
- TypeScript class [`DatatableHint`](https://github.com/stencila/stencila/blob/main/ts/src/types/DatatableHint.ts)

***

This documentation was generated from [`DatatableHint.yaml`](https://github.com/stencila/stencila/blob/main/schema/DatatableHint.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
