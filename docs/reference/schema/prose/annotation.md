---
title: Annotation
description: Annotated content.
config:
  publish:
    ghost:
      type: post
      slug: annotation
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

# Properties

The `Annotation` type has these properties:

| Name         | Description                          | Type                                                                | Inherited from                                                     | `JSON-LD @id`                        | Aliases |
| ------------ | ------------------------------------ | ------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------ | ------- |
| `id`         | The identifier for this item.        | [`String`](https://stencila.ghost.io/docs/reference/schema/string)  | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -       |
| `content`    | The content that is marked.          | [`Inline`](https://stencila.ghost.io/docs/reference/schema/inline)* | [`Mark`](https://stencila.ghost.io/docs/reference/schema/mark)     | `stencila:content`                   | -       |
| `annotation` | The annotation, usually a paragraph. | [`Block`](https://stencila.ghost.io/docs/reference/schema/block)*   | -                                                                  | `stencila:annotation`                | -       |

# Related

The `Annotation` type is related to these types:

- Parents: [`Mark`](https://stencila.ghost.io/docs/reference/schema/mark)
- Children: none

# Formats

The `Annotation` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                       | Encoding     | Decoding   | Support                                                                                                        | Notes |
| ---------------------------------------------------------------------------- | ------------ | ---------- | -------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)        | 🟢 No loss    |            |                                                                                                                |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                | 🔷 Low loss   |            |                                                                                                                |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                |              |            | Encoded as [`<annotation>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/annotation.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)              | ⚠️ High loss |            | Encoded as `=={{content}}==`                                                                                   |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)    | ⚠️ High loss |            |                                                                                                                |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)      | ⚠️ High loss |            |                                                                                                                |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)       | ⚠️ High loss |            |                                                                                                                |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)        | ⚠️ High loss |            |                                                                                                                |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)              | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                  | 🔷 Low loss   |            |                                                                                                                |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)          | ⚠️ High loss |            |                                                                                                                |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)              | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx) | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)     | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                  | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)        | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)              | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)           | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)      | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)     | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)       | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)        | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)      |              |            |                                                                                                                |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)  |              |            |                                                                                                                |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)              | 🔷 Low loss   |            |                                                                                                                |

# Bindings

The `Annotation` type is represented in:

- [JSON-LD](https://stencila.org/Annotation.jsonld)
- [JSON Schema](https://stencila.org/Annotation.schema.json)
- Python class [`Annotation`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/annotation.py)
- Rust struct [`Annotation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/annotation.rs)
- TypeScript class [`Annotation`](https://github.com/stencila/stencila/blob/main/ts/src/types/Annotation.ts)

# Source

This documentation was generated from [`Annotation.yaml`](https://github.com/stencila/stencila/blob/main/schema/Annotation.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
