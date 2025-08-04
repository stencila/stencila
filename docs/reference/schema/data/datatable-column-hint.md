---
title: Datatable Column Hint
description: A hint to the type and values in a `DatatableColumn`.
config:
  publish:
    ghost:
      type: post
      slug: datatable-column-hint
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Data
---

# Properties

The `DatatableColumnHint` type has these properties:

| Name       | Description                                | Type                                                                     | Inherited from                                                     | `JSON-LD @id`                        | Aliases                  |
| ---------- | ------------------------------------------ | ------------------------------------------------------------------------ | ------------------------------------------------------------------ | ------------------------------------ | ------------------------ |
| `id`       | The identifier for this item.              | [`String`](https://stencila.ghost.io/docs/reference/schema/string)       | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -                        |
| `name`     | The name of the column.                    | [`String`](https://stencila.ghost.io/docs/reference/schema/string)       | -                                                                  | `stencila:name`                      | -                        |
| `itemType` | The type of items in the column.           | [`String`](https://stencila.ghost.io/docs/reference/schema/string)       | -                                                                  | `stencila:itemType`                  | `item-type`, `item_type` |
| `minimum`  | The minimum value in the column.           | [`Primitive`](https://stencila.ghost.io/docs/reference/schema/primitive) | -                                                                  | `stencila:minimum`                   | -                        |
| `maximum`  | The maximum value in the column.           | [`Primitive`](https://stencila.ghost.io/docs/reference/schema/primitive) | -                                                                  | `stencila:maximum`                   | -                        |
| `nulls`    | The number of `Null` values in the column. | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)     | -                                                                  | `stencila:nulls`                     | -                        |

# Related

The `DatatableColumnHint` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `DatatableColumnHint` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |              |              |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | ⚠️ High loss |              |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | ⚠️ High loss |              |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | ⚠️ High loss |              |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | ⚠️ High loss |              |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | ⚠️ High loss |              |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |         |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss   |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |         |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |         |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |         |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |         |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |         |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |         |

# Bindings

The `DatatableColumnHint` type is represented in:

- [JSON-LD](https://stencila.org/DatatableColumnHint.jsonld)
- [JSON Schema](https://stencila.org/DatatableColumnHint.schema.json)
- Python class [`DatatableColumnHint`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/datatable_column_hint.py)
- Rust struct [`DatatableColumnHint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/datatable_column_hint.rs)
- TypeScript class [`DatatableColumnHint`](https://github.com/stencila/stencila/blob/main/ts/src/types/DatatableColumnHint.ts)

# Source

This documentation was generated from [`DatatableColumnHint.yaml`](https://github.com/stencila/stencila/blob/main/schema/DatatableColumnHint.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
