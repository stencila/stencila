---
title: Cite Group
description: A group of `Cite` nodes.
config:
  publish:
    ghost:
      type: post
      slug: cite-group
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

This type allows you to group associated citations together.
When some content in a [`Creative Work`](./CreativeWork) cites more than one
reference for a particular piece of text, use a `CiteGroup` to encapsulate
multiple [`Cite`](./Cite) nodes.

At present we do not give a `citationMode` property to a `CiteGroup` since
they will almost always be parenthetical as opposed to narrative.
In other words, it usually only makes sense for individual `Cite` nodes to be
narrative (although they may be connected together within `content` using words
such as "and").


# Properties

The `CiteGroup` type has these properties:

| Name    | Description                                                        | Type                                                               | Inherited from                                                     | `JSON-LD @id`                                                  | Aliases |
| ------- | ------------------------------------------------------------------ | ------------------------------------------------------------------ | ------------------------------------------------------------------ | -------------------------------------------------------------- | ------- |
| `id`    | The identifier for this item.                                      | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                           | -       |
| `items` | One or more `Cite`s to be referenced in the same surrounding text. | [`Cite`](https://stencila.ghost.io/docs/reference/schema/cite)*    | -                                                                  | [`schema:itemListElement`](https://schema.org/itemListElement) | `item`  |

# Related

The `CiteGroup` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `CiteGroup` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

# Bindings

The `CiteGroup` type is represented in:

- [JSON-LD](https://stencila.org/CiteGroup.jsonld)
- [JSON Schema](https://stencila.org/CiteGroup.schema.json)
- Python class [`CiteGroup`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/cite_group.py)
- Rust struct [`CiteGroup`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/cite_group.rs)
- TypeScript class [`CiteGroup`](https://github.com/stencila/stencila/blob/main/ts/src/types/CiteGroup.ts)

# Source

This documentation was generated from [`CiteGroup.yaml`](https://github.com/stencila/stencila/blob/main/schema/CiteGroup.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
