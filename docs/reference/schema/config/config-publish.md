---
title: Config Publish
description: Publishing options.
config:
  publish:
    ghost:
      type: post
      slug: config-publish
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Config
---

# Properties

The `ConfigPublish` type has these properties:

| Name     | Description                | Type                                                                                           | Inherited from | `JSON-LD @id` | Aliases |
| -------- | -------------------------- | ---------------------------------------------------------------------------------------------- | -------------- | ------------- | ------- |
| `ghost`  | Ghost publishing options.  | [`ConfigPublishGhost`](https://stencila.ghost.io/docs/reference/schema/config-publish-ghost)   | -              | ``            | -       |
| `zenodo` | Zenodo publishing options. | [`ConfigPublishZenodo`](https://stencila.ghost.io/docs/reference/schema/config-publish-zenodo) | -              | ``            | -       |

# Related

The `ConfigPublish` type is related to these types:

- Parents: None
- Children: none

# Formats

The `ConfigPublish` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                               | Encoding | Decoding | Support | Notes |
| ------------------------------------------------------------------------------------ | -------- | -------- | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)                |          |          |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                        |          |          |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                        |          |          |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                      |          |          |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)            |          |          |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)              |          |          |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)               |          |          |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)                |          |          |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                      |          |          |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                          |          |          |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                  |          |          |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                      |          |          |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)         |          |          |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)             |          |          |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                          |          |          |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                        |          |          |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)                |          |          |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                      |          |          |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                   |          |          |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                        |          |          |         |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)              |          |          |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                        |          |          |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)             |          |          |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)               |          |          |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)                |          |          |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)              |          |          |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)          |          |          |         |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoap) |          |          |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                      |          |          |         |

# Bindings

The `ConfigPublish` type is represented in:

- [JSON-LD](https://stencila.org/ConfigPublish.jsonld)
- [JSON Schema](https://stencila.org/ConfigPublish.schema.json)
- Python class [`ConfigPublish`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/config_publish.py)
- Rust struct [`ConfigPublish`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/config_publish.rs)
- TypeScript class [`ConfigPublish`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConfigPublish.ts)

# Source

This documentation was generated from [`ConfigPublish.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConfigPublish.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
