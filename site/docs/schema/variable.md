---
title: Variable
description: A variable representing a name / value pair.
---

This type is marked as unstable and is subject to change.

# Properties

The `Variable` type has these properties:

| Name                  | Description                                                                           | Type                    | Inherited from          | `JSON-LD @id`                                                          | Aliases                                        |
| --------------------- | ------------------------------------------------------------------------------------- | ----------------------- | ----------------------- | ---------------------------------------------------------------------- | ---------------------------------------------- |
| `id`                  | The identifier for this item.                                                         | [`String`](./string.md) | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)                                   | -                                              |
| `name`                | The name of the variable.                                                             | [`String`](./string.md) | -                       | [`schema:name`](https://schema.org/name)                               | -                                              |
| `programmingLanguage` | The programming language that the variable is defined in e.g. Python, JSON.           | [`String`](./string.md) | -                       | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | `programming-language`, `programming_language` |
| `nativeType`          | The native type of the variable e.g. `float`, `datetime.datetime`, `pandas.DataFrame` | [`String`](./string.md) | -                       | `stencila:nativeType`                                                  | `native-type`, `native_type`                   |
| `nodeType`            | The Stencila node type of the variable e.g. `Number`, `DateTime`, `Datatable`.        | [`String`](./string.md) | -                       | `stencila:nodeType`                                                    | `node-type`, `node_type`                       |
| `value`               | The value of the variable.                                                            | [`Node`](./node.md)     | -                       | [`schema:value`](https://schema.org/value)                             | -                                              |
| `hint`                | A hint to the value and/or structure of the variable.                                 | [`Hint`](./hint.md)     | -                       | `stencila:hint`                                                        | -                                              |
| `nativeHint`          | A textual hint to the value and/or structure of the variable.                         | [`String`](./string.md) | -                       | `stencila:nativeHint`                                                  | `native-hint`, `native_hint`                   |

# Related

The `Variable` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `Variable` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `Variable` type is represented in:

- [JSON-LD](https://stencila.org/Variable.jsonld)
- [JSON Schema](https://stencila.org/Variable.schema.json)
- Python class [`Variable`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/variable.py)
- Rust struct [`Variable`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/variable.rs)
- TypeScript class [`Variable`](https://github.com/stencila/stencila/blob/main/ts/src/types/Variable.ts)

# Source

This documentation was generated from [`Variable.yaml`](https://github.com/stencila/stencila/blob/main/schema/Variable.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
