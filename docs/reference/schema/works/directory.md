---
title: Directory
description: A directory on the file system.
config:
  publish:
    ghost:
      type: post
      slug: directory
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Works
---

Previously this type extended `Collection` (which in turn extends `CreativeWork`).
However, to avoid consuming more memory that necessary when creating directory listings
with many directories, it now extends `Entity`.


# Properties

The `Directory` type has these properties:

| Name    | Description                                                     | Type                                                                                                                                          | Inherited from                                                     | `JSON-LD @id`                                    | Aliases            |
| ------- | --------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------ | ------------------ |
| `id`    | The identifier for this item.                                   | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                            | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)             | -                  |
| `name`  | The name of the directory.                                      | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                            | -                                                                  | [`schema:name`](https://schema.org/name)         | -                  |
| `path`  | The path (absolute or relative) of the file on the file system. | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                            | -                                                                  | `stencila:path`                                  | -                  |
| `parts` | The files and other directories within this directory.          | ([`File`](https://stencila.ghost.io/docs/reference/schema/file) \| [`Directory`](https://stencila.ghost.io/docs/reference/schema/directory))* | -                                                                  | [`schema:hasParts`](https://schema.org/hasParts) | `hasParts`, `part` |

# Related

The `Directory` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `Directory` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)      |              | 🟢 No loss  |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)  |              |            |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)              | 🔷 Low loss   |            |         |

# Bindings

The `Directory` type is represented in:

- [JSON-LD](https://stencila.org/Directory.jsonld)
- [JSON Schema](https://stencila.org/Directory.schema.json)
- Python class [`Directory`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/directory.py)
- Rust struct [`Directory`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/directory.rs)
- TypeScript class [`Directory`](https://github.com/stencila/stencila/blob/main/ts/src/types/Directory.ts)

# Source

This documentation was generated from [`Directory.yaml`](https://github.com/stencila/stencila/blob/main/schema/Directory.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
