---
title: Excerpt
description: An excerpt from a `CreativeWork`.
config:
  publish:
    ghost:
      type: post
      slug: excerpt
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Other
---

# Properties

The `Excerpt` type has these properties:

| Name            | Description                                                          | Type                                                                     | Inherited from                                                     | `JSON-LD @id`                        | Aliases                            |
| --------------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------ | ------------------------------------------------------------------ | ------------------------------------ | ---------------------------------- |
| `id`            | The identifier for this item.                                        | [`String`](https://stencila.ghost.io/docs/reference/schema/string)       | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -                                  |
| `source`        | A `Reference` to the `CreativeWork` that the excerpt was taken from. | [`Reference`](https://stencila.ghost.io/docs/reference/schema/reference) | -                                                                  | `stencila:source`                    | -                                  |
| `nodePath`      | A `Reference` to the `CreativeWork` that the excerpt was taken from. | [`String`](https://stencila.ghost.io/docs/reference/schema/string)       | -                                                                  | `stencila:nodePath`                  | `node-path`, `node_path`           |
| `nodeAncestors` | The route to the node that was excerpted including the .             | [`String`](https://stencila.ghost.io/docs/reference/schema/string)       | -                                                                  | `stencila:nodeAncestors`             | `node-ancestors`, `node_ancestors` |
| `nodeType`      | The type of the node that was excerpted.                             | [`String`](https://stencila.ghost.io/docs/reference/schema/string)       | -                                                                  | `stencila:nodeType`                  | `node-type`, `node_type`           |
| `content`       | The excerpted content.                                               | [`Block`](https://stencila.ghost.io/docs/reference/schema/block)*        | -                                                                  | `stencila:content`                   | -                                  |

# Related

The `Excerpt` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `Excerpt` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                            | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                    |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |                                    |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |              |              |                                    |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | ⚠️ High loss |              | Encoded using implemented function |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | ⚠️ High loss |              |                                    |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | ⚠️ High loss |              |                                    |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | ⚠️ High loss |              |                                    |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | ⚠️ High loss |              |                                    |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                    |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                    |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |                                    |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |                                    |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                    |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                    |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                    |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                    |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                    |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |                                    |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                    |

# Bindings

The `Excerpt` type is represented in:

- [JSON-LD](https://stencila.org/Excerpt.jsonld)
- [JSON Schema](https://stencila.org/Excerpt.schema.json)
- Python class [`Excerpt`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/excerpt.py)
- Rust struct [`Excerpt`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/excerpt.rs)
- TypeScript class [`Excerpt`](https://github.com/stencila/stencila/blob/main/ts/src/types/Excerpt.ts)

# Source

This documentation was generated from [`Excerpt.yaml`](https://github.com/stencila/stencila/blob/main/schema/Excerpt.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
