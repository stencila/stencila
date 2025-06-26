---
title: Config Publish Zenodo
description: Zenodo publishing options.
config:
  publish:
    ghost:
      type: post
      slug: config-publish-zenodo
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Config
---

# Properties

The `ConfigPublishZenodo` type has these properties:

| Name           | Description                       | Type                                                                                                                   | Inherited from | `JSON-LD @id` | Aliases        |
| -------------- | --------------------------------- | ---------------------------------------------------------------------------------------------------------------------- | -------------- | ------------- | -------------- |
| `embargoed`    | The date of embargoed.            | [`Date`](https://stencila.ghost.io/docs/reference/schema/date)                                                         | -              | ``            | -              |
| `access_right` | The access right of the document. | [`ConfigPublishZenodoAccessRight`](https://stencila.ghost.io/docs/reference/schema/config-publish-zenodo-access-right) | -              | ``            | `access-right` |
| `notes`        | extra notes about deposition.     | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                     | -              | ``            | -              |
| `method`       | The methodology of the study.     | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                     | -              | ``            | -              |

# Related

The `ConfigPublishZenodo` type is related to these types:

- Parents: None
- Children: none

# Formats

The `ConfigPublishZenodo` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `ConfigPublishZenodo` type is represented in:

- [JSON-LD](https://stencila.org/ConfigPublishZenodo.jsonld)
- [JSON Schema](https://stencila.org/ConfigPublishZenodo.schema.json)
- Python class [`ConfigPublishZenodo`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/config_publish_zenodo.py)
- Rust struct [`ConfigPublishZenodo`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/config_publish_zenodo.rs)
- TypeScript class [`ConfigPublishZenodo`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConfigPublishZenodo.ts)

# Source

This documentation was generated from [`ConfigPublishZenodo.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConfigPublishZenodo.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
