---
title: Datatable Column
description: A column of data within a `Datatable`.
---

# Properties

The `DatatableColumn` type has these properties:

| Name        | Description                                          | Type                                     | Inherited from          | `JSON-LD @id`                        | Aliases |
| ----------- | ---------------------------------------------------- | ---------------------------------------- | ----------------------- | ------------------------------------ | ------- |
| `id`        | The identifier for this item.                        | [`String`](./string.md)                  | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id) | -       |
| `name`      | The name of the column.                              | [`String`](./string.md)                  | -                       | `stencila:name`                      | -       |
| `values`    | The data values of the column.                       | [`Primitive`](./primitive.md)*           | -                       | `stencila:values`                    | `value` |
| `validator` | The validator to use to validate data in the column. | [`ArrayValidator`](./array-validator.md) | -                       | `stencila:validator`                 | -       |

# Related

The `DatatableColumn` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `DatatableColumn` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support | Notes |
| ------------------------------------------------ | ------------ | ------------ | ------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |         |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |         |
| [JATS](../formats/jats.md)                       |              |              |         |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              |         |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |         |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |         |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |         |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |         |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |         |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |         |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |         |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |         |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |         |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |         |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |         |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |         |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |         |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |         |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |         |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |         |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |         |
| [CSL-JSON](../formats/csl.md)                    |              |              |         |
| [Citation File Format](../formats/cff.md)        |              |              |         |
| [CSV](../formats/csv.md)                         |              |              |         |
| [TSV](../formats/tsv.md)                         |              |              |         |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |         |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |         |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |         |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |         |
| [Directory](../formats/directory.md)             |              |              |         |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |         |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |         |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |         |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |         |
| [Email HTML](../formats/email.html.md)           |              |              |         |
| [MJML](../formats/mjml.md)                       |              |              |         |

# Bindings

The `DatatableColumn` type is represented in:

- [JSON-LD](https://stencila.org/DatatableColumn.jsonld)
- [JSON Schema](https://stencila.org/DatatableColumn.schema.json)
- Python class [`DatatableColumn`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/datatable_column.py)
- Rust struct [`DatatableColumn`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/datatable_column.rs)
- TypeScript class [`DatatableColumn`](https://github.com/stencila/stencila/blob/main/ts/src/types/DatatableColumn.ts)

# Source

This documentation was generated from [`DatatableColumn.yaml`](https://github.com/stencila/stencila/blob/main/schema/DatatableColumn.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
