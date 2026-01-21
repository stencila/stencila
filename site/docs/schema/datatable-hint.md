---
title: Datatable Hint
description: A hint to the structure of a table of data.
---

# Properties

The `DatatableHint` type has these properties:

| Name      | Description                     | Type                                                 | Inherited from          |
| --------- | ------------------------------- | ---------------------------------------------------- | ----------------------- |
| `id`      | The identifier for this item.   | [`String`](./string.md)                              | [`Entity`](./entity.md) |
| `rows`    | The number of rows of data.     | [`Integer`](./integer.md)                            | -                       |
| `columns` | A hint for each column of data. | [`DatatableColumnHint`](./datatable-column-hint.md)* | -                       |

# Related

The `DatatableHint` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `DatatableHint` type is represented in:

- [JSON-LD](https://stencila.org/DatatableHint.jsonld)
- [JSON Schema](https://stencila.org/DatatableHint.schema.json)
- Python class [`DatatableHint`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/datatable_hint.py)
- Rust struct [`DatatableHint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/datatable_hint.rs)
- TypeScript class [`DatatableHint`](https://github.com/stencila/stencila/blob/main/ts/src/types/DatatableHint.ts)

# Source

This documentation was generated from [`DatatableHint.yaml`](https://github.com/stencila/stencila/blob/main/schema/DatatableHint.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
