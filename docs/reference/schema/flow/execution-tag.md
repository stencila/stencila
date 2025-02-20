---
title: Execution Tag
description: A tag on code that affects its execution.
config:
  publish:
    ghost:
      type: page
      slug: execution-tag
      state: publish
      tags:
      - '#schema'
      - '#doc'
      - Flow
---

## Properties

The `ExecutionTag` type has these properties:

| Name       | Description                               | Type                                                                 | Inherited from                                                     | `JSON-LD @id`                              | Aliases                  |
| ---------- | ----------------------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------ | ------------------------ |
| `id`       | The identifier for this item.             | [`String`](https://stencila.ghost.io/docs/reference/schema/string)   | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)       | -                        |
| `name`     | The name of the tag                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)   | -                                                                  | [`schema:name`](https://schema.org/name)   | -                        |
| `value`    | The value of the tag                      | [`String`](https://stencila.ghost.io/docs/reference/schema/string)   | -                                                                  | [`schema:value`](https://schema.org/value) | -                        |
| `isGlobal` | Whether the tag is global to the document | [`Boolean`](https://stencila.ghost.io/docs/reference/schema/boolean) | -                                                                  | `stencila:isGlobal`                        | `is-global`, `is_global` |

## Related

The `ExecutionTag` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

## Formats

The `ExecutionTag` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                       | Encoding     | Decoding   | Support | Notes |
| ---------------------------------------------------------------------------- | ------------ | ---------- | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)        | 🟢 No loss    |            |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                | 🔷 Low loss   |            |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                |              |            |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)              | ⚠️ High loss |            |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)    | ⚠️ High loss |            |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)      | ⚠️ High loss |            |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)       | ⚠️ High loss |            |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)        | ⚠️ High loss |            |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)              | 🔷 Low loss   | 🔷 Low loss |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                  | 🔷 Low loss   |            |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)          | ⚠️ High loss |            |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)              | 🔷 Low loss   | 🔷 Low loss |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx) | 🔷 Low loss   | 🔷 Low loss |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)     | 🔷 Low loss   | 🔷 Low loss |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                  | 🔷 Low loss   | 🔷 Low loss |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                | 🟢 No loss    | 🟢 No loss  |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)        | 🟢 No loss    | 🟢 No loss  |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)              | 🟢 No loss    | 🟢 No loss  |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)           | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR+Zstandard](https://stencila.ghost.io/docs/reference/formats/cbor.zstd) | 🟢 No loss    | 🟢 No loss  |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                | 🟢 No loss    | 🟢 No loss  |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)     | 🔷 Low loss   | 🔷 Low loss |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)       | 🔷 Low loss   | 🔷 Low loss |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)        | 🔷 Low loss   | 🔷 Low loss |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)      |              |            |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)  |              |            |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)              | 🔷 Low loss   |            |         |

## Bindings

The `ExecutionTag` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionTag.jsonld)
- [JSON Schema](https://stencila.org/ExecutionTag.schema.json)
- Python class [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/execution_tag.py)
- Rust struct [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_tag.rs)
- TypeScript class [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionTag.ts)

## Source

This documentation was generated from [`ExecutionTag.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionTag.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
