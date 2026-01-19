---
title: Config Publish Zenodo
description: Zenodo publishing options.
---

# Properties

The `ConfigPublishZenodo` type has these properties:

| Name           | Description                       | Type                                                                        | Inherited from | `JSON-LD @id` | Aliases        |
| -------------- | --------------------------------- | --------------------------------------------------------------------------- | -------------- | ------------- | -------------- |
| `embargoed`    | The date of embargoed.            | [`Date`](./date.md)                                                         | -              | ``            | -              |
| `access_right` | The access right of the document. | [`ConfigPublishZenodoAccessRight`](./config-publish-zenodo-access-right.md) | -              | ``            | `access-right` |
| `notes`        | extra notes about deposition.     | [`String`](./string.md)                                                     | -              | ``            | -              |
| `method`       | The methodology of the study.     | [`String`](./string.md)                                                     | -              | ``            | -              |

# Related

The `ConfigPublishZenodo` type is related to these types:

- Parents: None
- Children: none

# Formats

The `ConfigPublishZenodo` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `ConfigPublishZenodo` type is represented in:

- [JSON-LD](https://stencila.org/ConfigPublishZenodo.jsonld)
- [JSON Schema](https://stencila.org/ConfigPublishZenodo.schema.json)
- Python class [`ConfigPublishZenodo`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/config_publish_zenodo.py)
- Rust struct [`ConfigPublishZenodo`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/config_publish_zenodo.rs)
- TypeScript class [`ConfigPublishZenodo`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConfigPublishZenodo.ts)

# Source

This documentation was generated from [`ConfigPublishZenodo.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConfigPublishZenodo.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
