---
title: Config Publish Ghost
description: Ghost publishing options.
config:
  publish:
    ghost:
      type: page
      slug: config-publish-ghost
      state: publish
      tags:
      - '#schema'
      - '#doc'
      - Config
---

## Properties

The `ConfigPublishGhost` type has these properties:

| Name       | Description                                          | Type                                                                                                    | Inherited from | `JSON-LD @id` | Aliases |
| ---------- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------- | -------------- | ------------- | ------- |
| `slug`     | The URL slug for the page or post.                   | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                      | -              | ``            | -       |
| `featured` | Whether the page or post is featured.                | [`Boolean`](https://stencila.ghost.io/docs/reference/schema/boolean)                                    | -              | ``            | -       |
| `schedule` | The date that the page or post is to be published.   | [`Date`](https://stencila.ghost.io/docs/reference/schema/date)                                          | -              | ``            | -       |
| `state`    | the state of the page or post eg draft or published. | [`ConfigPublishGhostState`](https://stencila.ghost.io/docs/reference/schema/config-publish-ghost-state) | -              | ``            | -       |
| `tags`     | ghost tags.                                          | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                     | -              | ``            | `tag`   |

## Related

The `ConfigPublishGhost` type is related to these types:

- Parents: None
- Children: none

## Formats

The `ConfigPublishGhost` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                       | Encoding | Decoding | Support | Notes |
| ---------------------------------------------------------------------------- | -------- | -------- | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)        |          |          |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                |          |          |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                |          |          |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)              |          |          |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)    |          |          |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)      |          |          |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)       |          |          |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)        |          |          |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)              |          |          |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                  |          |          |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)          |          |          |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)              |          |          |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx) |          |          |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)     |          |          |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                  |          |          |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                |          |          |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)        |          |          |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)              |          |          |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)           |          |          |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                |          |          |         |
| [CBOR+Zstandard](https://stencila.ghost.io/docs/reference/formats/cbor.zstd) |          |          |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                |          |          |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)     |          |          |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)       |          |          |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)        |          |          |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)      |          |          |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)  |          |          |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)              |          |          |         |

## Bindings

The `ConfigPublishGhost` type is represented in:

- [JSON-LD](https://stencila.org/ConfigPublishGhost.jsonld)
- [JSON Schema](https://stencila.org/ConfigPublishGhost.schema.json)
- Python class [`ConfigPublishGhost`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/config_publish_ghost.py)
- Rust struct [`ConfigPublishGhost`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/config_publish_ghost.rs)
- TypeScript class [`ConfigPublishGhost`](https://github.com/stencila/stencila/blob/main/ts/src/types/ConfigPublishGhost.ts)

## Source

This documentation was generated from [`ConfigPublishGhost.yaml`](https://github.com/stencila/stencila/blob/main/schema/ConfigPublishGhost.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
