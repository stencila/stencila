---
title: Config Models
description: Model selection and execution options.
---

# Properties

The `ConfigModels` type has these properties:

| Name              | Description                                                            | Type                                       | Inherited from | `JSON-LD @id` | Aliases                                                                             |
| ----------------- | ---------------------------------------------------------------------- | ------------------------------------------ | -------------- | ------------- | ----------------------------------------------------------------------------------- |
| `executeContent`  | Automatically execute generated content.                               | [`Boolean`](./boolean.md)                  | -              | ``            | `execute-content`, `execute_content`                                                |
| `executionBounds` | The execution boundaries on model generated code.                      | [`ExecutionBounds`](./execution-bounds.md) | -              | ``            | `execution-bounds`, `execution_bounds`                                              |
| `maximumRetries`  | When executing model generated content, the maximum number of retries. | [`Number`](./number.md)                    | -              | ``            | `max-retries`, `maximum-retries`, `execution-retries`, `retries`, `maximum_retries` |

# Related

The `ConfigModels` type is related to these types:

- Parents: None
- Children: none

# Formats

The `ConfigModels` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding | Decoding | Support | Notes |
| ------------------------------------------------ | -------- | -------- | ------- | ----- |
| [DOM HTML](../formats/dom.html.md)               |          |          |         |
| [HTML](../formats/html.md)                       |          |          |         |
| [JATS](../formats/jats.md)                       |          |          |         |
| [Markdown](../formats/md.md)                     |          |          |         |
| [Stencila Markdown](../formats/smd.md)           |          |          |         |
| [Quarto Markdown](../formats/qmd.md)             |          |          |         |
| [MyST Markdown](../formats/myst.md)              |          |          |         |
| [LLM Markdown](../formats/llmd.md)               |          |          |         |
| [LaTeX](../formats/latex.md)                     |          |          |         |
| [R+LaTeX](../formats/rnw.md)                     |          |          |         |
| [PDF](../formats/pdf.md)                         |          |          |         |
| [Plain text](../formats/text.md)                 |          |          |         |
| [IPYNB](../formats/ipynb.md)                     |          |          |         |
| [Microsoft Word](../formats/docx.md)             |          |          |         |
| [OpenDocument Text](../formats/odt.md)           |          |          |         |
| [TeX](../formats/tex.md)                         |          |          |         |
| [JSON](../formats/json.md)                       |          |          |         |
| [JSON+Zip](../formats/json.zip.md)               |          |          |         |
| [JSON5](../formats/json5.md)                     |          |          |         |
| [JSON-LD](../formats/jsonld.md)                  |          |          |         |
| [CBOR](../formats/cbor.md)                       |          |          |         |
| [CBOR+Zstd](../formats/czst.md)                  |          |          |         |
| [YAML](../formats/yaml.md)                       |          |          |         |
| [Lexical JSON](../formats/lexical.md)            |          |          |         |
| [Koenig JSON](../formats/koenig.md)              |          |          |         |
| [Pandoc AST](../formats/pandoc.md)               |          |          |         |
| [CSL-JSON](../formats/csl.md)                    |          |          |         |
| [Citation File Format](../formats/cff.md)        |          |          |         |
| [CSV](../formats/csv.md)                         |          |          |         |
| [TSV](../formats/tsv.md)                         |          |          |         |
| [Microsoft Excel](../formats/xlsx.md)            |          |          |         |
| [Microsoft Excel (XLS)](../formats/xls.md)       |          |          |         |
| [OpenDocument Spreadsheet](../formats/ods.md)    |          |          |         |
| [PNG](../formats/png.md)                         |          |          |         |
| [Directory](../formats/directory.md)             |          |          |         |
| [Stencila Web Bundle](../formats/swb.md)         |          |          |         |
| [Meca](../formats/meca.md)                       |          |          |         |
| [PubMed Central OA Package](../formats/pmcoa.md) |          |          |         |
| [Debug](../formats/debug.md)                     |          |          |         |
| [Email HTML](../formats/email.html.md)           |          |          |         |
| [MJML](../formats/mjml.md)                       |          |          |         |

# Bindings

The `ConfigModels` type is represented in:

- [JSON-LD](https://stencila.org/ConfigModels.jsonld)
- [JSON Schema](https://stencila.org/ConfigModels.schema.json)
- Python class [`ConfigModels`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/config_models.py)
- Rust struct [`ConfigModels`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/config_models.rs)
- TypeScript class [`ConfigModels`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConfigModels.ts)

# Source

This documentation was generated from [`ConfigModels.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConfigModels.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
