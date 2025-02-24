---
title: Integer
description: A value that is a integer.
config:
  publish:
    ghost:
      type: post
      slug: integer
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Data
---

# Formats

The `Integer` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                       | Encoding   | Decoding   | Support | Notes |
| ---------------------------------------------------------------------------- | ---------- | ---------- | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)        | 🟢 No loss  |            |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                | 🔷 Low loss |            |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                | 🔷 Low loss |            |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)              | 🔷 Low loss | 🔷 Low loss |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)    | 🔷 Low loss | 🔷 Low loss |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)      | 🔷 Low loss | 🔷 Low loss |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)       | 🔷 Low loss | 🔷 Low loss |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)        | 🔷 Low loss | 🔷 Low loss |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)              | 🔷 Low loss | 🔷 Low loss |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                  | 🔷 Low loss |            |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)          | 🔷 Low loss |            |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)              | 🔷 Low loss | 🔷 Low loss |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx) | 🔷 Low loss | 🔷 Low loss |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)     | 🔷 Low loss | 🔷 Low loss |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                  | 🔷 Low loss | 🔷 Low loss |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                | 🟢 No loss  | 🟢 No loss  |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)        | 🟢 No loss  | 🟢 No loss  |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)              | 🟢 No loss  | 🟢 No loss  |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)           | 🟢 No loss  | 🟢 No loss  |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                | 🟢 No loss  | 🟢 No loss  |         |
| [CBOR+Zstandard](https://stencila.ghost.io/docs/reference/formats/cbor.zstd) | 🟢 No loss  | 🟢 No loss  |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                | 🟢 No loss  | 🟢 No loss  |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)     | 🔷 Low loss | 🔷 Low loss |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)       | 🔷 Low loss | 🔷 Low loss |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)        | 🔷 Low loss | 🔷 Low loss |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)      |            |            |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)  |            |            |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)              | 🔷 Low loss |            |         |

# Bindings

The `Integer` type is represented in:

- [JSON-LD](https://stencila.org/Integer.jsonld)
- [JSON Schema](https://stencila.org/Integer.schema.json)
- Python type [`Integer`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/integer.py)
- Rust type [`Integer`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/integer.rs)
- TypeScript type [`Integer`](https://github.com/stencila/stencila/blob/main/ts/src/types/Integer.ts)

# Source

This documentation was generated from [`Integer.yaml`](https://github.com/stencila/stencila/blob/main/schema/Integer.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
