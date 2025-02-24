---
title: Thematic Break
description: A thematic break, such as a scene change in a story, a transition to another topic, or a new document.
config:
  publish:
    ghost:
      type: post
      slug: thematic-break
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

# Properties

The `ThematicBreak` type has these properties:

| Name | Description                   | Type                                                               | Inherited from                                                     | `JSON-LD @id`                        | Aliases |
| ---- | ----------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------ | ------------------------------------ | ------- |
| `id` | The identifier for this item. | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -       |

# Related

The `ThematicBreak` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `ThematicBreak` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                       | Encoding     | Decoding   | Support                                                                                        | Notes |
| ---------------------------------------------------------------------------- | ------------ | ---------- | ---------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)        | 🟢 No loss    |            |                                                                                                |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                | 🟢 No loss    |            | Encoded as [`<hr>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hr)              |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                | 🟢 No loss    | 🟢 No loss  | Encoded as [`<hr>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/hr.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)              | 🟢 No loss    | 🟢 No loss  | Encoded as `***\n\n`                                                                           |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)    | 🟢 No loss    | 🟢 No loss  |                                                                                                |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)      | 🟢 No loss    | 🟢 No loss  |                                                                                                |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)       | 🟢 No loss    | 🟢 No loss  |                                                                                                |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)        | 🟢 No loss    | 🟢 No loss  |                                                                                                |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)              | 🔷 Low loss   | 🔷 Low loss |                                                                                                |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                  | 🔷 Low loss   |            |                                                                                                |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)          | ⚠️ High loss |            |                                                                                                |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)              | 🔷 Low loss   | 🔷 Low loss |                                                                                                |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx) | 🔷 Low loss   | 🔷 Low loss |                                                                                                |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)     | 🔷 Low loss   | 🔷 Low loss |                                                                                                |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                  | 🔷 Low loss   | 🔷 Low loss |                                                                                                |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                | 🟢 No loss    | 🟢 No loss  |                                                                                                |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)        | 🟢 No loss    | 🟢 No loss  |                                                                                                |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)              | 🟢 No loss    | 🟢 No loss  |                                                                                                |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)           | 🟢 No loss    | 🟢 No loss  |                                                                                                |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                | 🟢 No loss    | 🟢 No loss  |                                                                                                |
| [CBOR+Zstandard](https://stencila.ghost.io/docs/reference/formats/cbor.zstd) | 🟢 No loss    | 🟢 No loss  |                                                                                                |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                | 🟢 No loss    | 🟢 No loss  |                                                                                                |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)     | 🔷 Low loss   | 🔷 Low loss |                                                                                                |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)       | 🔷 Low loss   | 🔷 Low loss |                                                                                                |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)        | 🔷 Low loss   | 🔷 Low loss |                                                                                                |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)      |              |            |                                                                                                |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)  |              |            |                                                                                                |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)              | 🔷 Low loss   |            |                                                                                                |

# Bindings

The `ThematicBreak` type is represented in:

- [JSON-LD](https://stencila.org/ThematicBreak.jsonld)
- [JSON Schema](https://stencila.org/ThematicBreak.schema.json)
- Python class [`ThematicBreak`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/thematic_break.py)
- Rust struct [`ThematicBreak`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/thematic_break.rs)
- TypeScript class [`ThematicBreak`](https://github.com/stencila/stencila/blob/main/ts/src/types/ThematicBreak.ts)

# Source

This documentation was generated from [`ThematicBreak.yaml`](https://github.com/stencila/stencila/blob/main/schema/ThematicBreak.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
