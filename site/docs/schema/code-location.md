---
title: Code Location
description: The location within some source code.
---

# Properties

The `CodeLocation` type has these properties:

| Name          | Description                                                        | Type                                       | Inherited from          | `JSON-LD @id`                        | Aliases                        |
| ------------- | ------------------------------------------------------------------ | ------------------------------------------ | ----------------------- | ------------------------------------ | ------------------------------ |
| `id`          | The identifier for this item.                                      | [`String`](./string.md)                    | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id) | -                              |
| `source`      | The source of the code, a file path, label or URL.                 | [`String`](./string.md)                    | -                       | `stencila:source`                    | -                              |
| `startLine`   | The 0-based index if the first line on which the error occurred.   | [`UnsignedInteger`](./unsigned-integer.md) | -                       | `stencila:startLine`                 | `start-line`, `start_line`     |
| `startColumn` | The 0-based index if the first column on which the error occurred. | [`UnsignedInteger`](./unsigned-integer.md) | -                       | `stencila:startColumn`               | `start-column`, `start_column` |
| `endLine`     | The 0-based index if the last line on which the error occurred.    | [`UnsignedInteger`](./unsigned-integer.md) | -                       | `stencila:endLine`                   | `end-line`, `end_line`         |
| `endColumn`   | The 0-based index if the last column on which the error occurred.  | [`UnsignedInteger`](./unsigned-integer.md) | -                       | `stencila:endColumn`                 | `end-column`, `end_column`     |

# Related

The `CodeLocation` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `CodeLocation` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `CodeLocation` type is represented in:

- [JSON-LD](https://stencila.org/CodeLocation.jsonld)
- [JSON Schema](https://stencila.org/CodeLocation.schema.json)
- Python class [`CodeLocation`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/code_location.py)
- Rust struct [`CodeLocation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_location.rs)
- TypeScript class [`CodeLocation`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeLocation.ts)

# Source

This documentation was generated from [`CodeLocation.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeLocation.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
