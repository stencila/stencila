---
title: Config Publish Ghost
description: Ghost publishing options.
---

# Properties

The `ConfigPublishGhost` type has these properties:

| Name       | Description                                          | Type                                                         | Inherited from | `JSON-LD @id` | Aliases |
| ---------- | ---------------------------------------------------- | ------------------------------------------------------------ | -------------- | ------------- | ------- |
| `slug`     | The URL slug for the page or post.                   | [`String`](./string.md)                                      | -              | ``            | -       |
| `featured` | Whether the page or post is featured.                | [`Boolean`](./boolean.md)                                    | -              | ``            | -       |
| `schedule` | The date that the page or post is to be published.   | [`Date`](./date.md)                                          | -              | ``            | -       |
| `state`    | the state of the page or post eg draft or published. | [`ConfigPublishGhostState`](./config-publish-ghost-state.md) | -              | ``            | -       |
| `tags`     | ghost tags.                                          | [`String`](./string.md)*                                     | -              | ``            | `tag`   |

# Related

The `ConfigPublishGhost` type is related to these types:

- Parents: None
- Children: none

# Formats

The `ConfigPublishGhost` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `ConfigPublishGhost` type is represented in:

- [JSON-LD](https://stencila.org/ConfigPublishGhost.jsonld)
- [JSON Schema](https://stencila.org/ConfigPublishGhost.schema.json)
- Python class [`ConfigPublishGhost`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/config_publish_ghost.py)
- Rust struct [`ConfigPublishGhost`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/config_publish_ghost.rs)
- TypeScript class [`ConfigPublishGhost`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConfigPublishGhost.ts)

# Source

This documentation was generated from [`ConfigPublishGhost.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConfigPublishGhost.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
