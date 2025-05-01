---
title: File
description: A file on the file system.
config:
  publish:
    ghost:
      type: post
      slug: file
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Works
---

Previously this type extended `CreativeWork`.
However, to avoid consuming more memory than necessary when creating directory listings
with many files, it now extends `Entity`.


# Properties

The `File` type has these properties:

| Name               | Description                                                    | Type                                                                                  | Inherited from                                                     | `JSON-LD @id`                                                | Aliases                                      |
| ------------------ | -------------------------------------------------------------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------ | -------------------------------------------- |
| `id`               | The identifier for this item.                                  | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                    | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                         | -                                            |
| `name`             | The name of the file.                                          | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                    | -                                                                  | [`schema:name`](https://schema.org/name)                     | -                                            |
| `path`             | The path (absolute or relative) of the file on the file system | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                    | -                                                                  | `stencila:path`                                              | -                                            |
| `mediaType`        | IANA media type (MIME type).                                   | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                    | -                                                                  | [`schema:encodingFormat`](https://schema.org/encodingFormat) | `encodingFormat`, `media-type`, `media_type` |
| `transferEncoding` | The encoding used for the context (e.g. base64, gz)            | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                    | -                                                                  | `stencila:transferEncoding`                                  | `transfer-encoding`, `transfer_encoding`     |
| `size`             | The size of the content in bytes                               | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer) | -                                                                  | [`schema:size`](https://schema.org/size)                     | -                                            |
| `content`          | The content of the file.                                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                    | -                                                                  | [`schema:content`](https://schema.org/content)               | -                                            |

# Related

The `File` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `File` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                               | Encoding     | Decoding   | Support | Notes |
| ------------------------------------------------------------------------------------ | ------------ | ---------- | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)                | 🟢 No loss    |            |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                        | 🔷 Low loss   |            |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                        |              |            |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                      | ⚠️ High loss |            |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)            | ⚠️ High loss |            |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)              | ⚠️ High loss |            |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)               | ⚠️ High loss |            |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)                | ⚠️ High loss |            |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                      | 🔷 Low loss   | 🔷 Low loss |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                          | 🔷 Low loss   |            |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                  | ⚠️ High loss |            |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                      | 🔷 Low loss   | 🔷 Low loss |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)         | 🔷 Low loss   | 🔷 Low loss |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)             | 🔷 Low loss   | 🔷 Low loss |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                          | 🔷 Low loss   | 🔷 Low loss |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                        | 🟢 No loss    | 🟢 No loss  |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)                | 🟢 No loss    | 🟢 No loss  |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                      | 🟢 No loss    | 🟢 No loss  |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                   | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                        | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)              | 🟢 No loss    | 🟢 No loss  |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                        | 🟢 No loss    | 🟢 No loss  |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)             | 🔷 Low loss   | 🔷 Low loss |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)               | 🔷 Low loss   | 🔷 Low loss |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)                | 🔷 Low loss   | 🔷 Low loss |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)              |              |            |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)          |              |            |         |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoap) |              | 🔷 Low loss |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                      | 🔷 Low loss   |            |         |

# Bindings

The `File` type is represented in:

- [JSON-LD](https://stencila.org/File.jsonld)
- [JSON Schema](https://stencila.org/File.schema.json)
- Python class [`File`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/file.py)
- Rust struct [`File`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/file.rs)
- TypeScript class [`File`](https://github.com/stencila/stencila/blob/main/ts/src/types/File.ts)

# Source

This documentation was generated from [`File.yaml`](https://github.com/stencila/stencila/blob/main/schema/File.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
