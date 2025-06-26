---
title: Config Models
description: Model selection and execution options.
config:
  publish:
    ghost:
      type: post
      slug: config-models
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Config
---

# Properties

The `ConfigModels` type has these properties:

| Name              | Description                                                            | Type                                                                                  | Inherited from | `JSON-LD @id` | Aliases                                                                             |
| ----------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------- | -------------- | ------------- | ----------------------------------------------------------------------------------- |
| `executeContent`  | Automatically execute generated content.                               | [`Boolean`](https://stencila.ghost.io/docs/reference/schema/boolean)                  | -              | ``            | `execute-content`, `execute_content`                                                |
| `executionBounds` | The execution boundaries on model generated code.                      | [`ExecutionBounds`](https://stencila.ghost.io/docs/reference/schema/execution-bounds) | -              | ``            | `execution-bounds`, `execution_bounds`                                              |
| `maximumRetries`  | When executing model generated content, the maximum number of retries. | [`Number`](https://stencila.ghost.io/docs/reference/schema/number)                    | -              | ``            | `max-retries`, `maximum-retries`, `execution-retries`, `retries`, `maximum_retries` |

# Related

The `ConfigModels` type is related to these types:

- Parents: None
- Children: none

# Formats

The `ConfigModels` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding | Decoding | Support | Notes |
| ----------------------------------------------------------------------------------- | -------- | -------- | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               |          |          |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       |          |          |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |          |          |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     |          |          |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           |          |          |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             |          |          |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              |          |          |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               |          |          |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     |          |          |         |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     |          |          |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         |          |          |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 |          |          |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     |          |          |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        |          |          |         |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |          |          |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            |          |          |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         |          |          |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       |          |          |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               |          |          |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     |          |          |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  |          |          |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       |          |          |         |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             |          |          |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       |          |          |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            |          |          |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              |          |          |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               |          |          |         |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         |          |          |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |          |          |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |          |          |         |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |          |          |         |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |          |          |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     |          |          |         |

# Bindings

The `ConfigModels` type is represented in:

- [JSON-LD](https://stencila.org/ConfigModels.jsonld)
- [JSON Schema](https://stencila.org/ConfigModels.schema.json)
- Python class [`ConfigModels`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/config_models.py)
- Rust struct [`ConfigModels`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/config_models.rs)
- TypeScript class [`ConfigModels`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConfigModels.ts)

# Source

This documentation was generated from [`ConfigModels.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConfigModels.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
