---
title: Citation Group
description: A group of `Citation` nodes.
config:
  publish:
    ghost:
      type: post
      slug: citation-group
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

This type allows you to group associated citations together.
When some content in a [`Creative Work`](./CreativeWork) Citations more than one
reference for a particular piece of text, use a `CitationGroup` to encapsulate
multiple [`Citation`](./Citation) nodes.

At present we do not give a `citationMode` property to a `CitationGroup` since
they will almost always be parenthetical as opposed to narrative.
In other words, it usually only makes sense for individual `Citation` nodes to be
narrative (although they may be connected together within `content` using words
such as "and").


# Properties

The `CitationGroup` type has these properties:

| Name      | Description                                                                 | Type                                                                    | Inherited from                                                     | `JSON-LD @id`                                                  | Aliases |
| --------- | --------------------------------------------------------------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------------------------- | ------- |
| `id`      | The identifier for this item.                                               | [`String`](https://stencila.ghost.io/docs/reference/schema/string)      | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                           | -       |
| `items`   | One or more `Citation`s to be referenced in the same surrounding text.      | [`Citation`](https://stencila.ghost.io/docs/reference/schema/citation)* | -                                                                  | [`schema:itemListElement`](https://schema.org/itemListElement) | `item`  |
| `content` | A rendering of the citation group using the citation style of the document. | [`Inline`](https://stencila.ghost.io/docs/reference/schema/inline)*     | -                                                                  | `stencila:content`                                             | -       |

# Related

The `CitationGroup` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `CitationGroup` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                            | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                    |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |                                    |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |              |              | Encoded using special function     |
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
| [Microsoft Word](https://stencila.ghost.io/docs/reference/formats/docx)             | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [OpenDocument Text](https://stencila.ghost.io/docs/reference/formats/odt)           | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/czst)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                    |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |                                    |
| [CSV](https://stencila.ghost.io/docs/reference/formats/csv)                         |              |              |                                    |
| [TSV](https://stencila.ghost.io/docs/reference/formats/tsv)                         |              |              |                                    |
| [Microsoft Excel](https://stencila.ghost.io/docs/reference/formats/xlsx)            |              |              |                                    |
| [Microsoft Excel (XLS)](https://stencila.ghost.io/docs/reference/formats/xls)       |              |              |                                    |
| [OpenDocument Spreadsheet](https://stencila.ghost.io/docs/reference/formats/ods)    |              |              |                                    |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                    |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                    |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                    |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                    |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              |              |                                    |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                    |

# Bindings

The `CitationGroup` type is represented in:

- [JSON-LD](https://stencila.org/CitationGroup.jsonld)
- [JSON Schema](https://stencila.org/CitationGroup.schema.json)
- Python class [`CitationGroup`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/citation_group.py)
- Rust struct [`CitationGroup`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/citation_group.rs)
- TypeScript class [`CitationGroup`](https://github.com/stencila/stencila/blob/main/ts/src/types/CitationGroup.ts)

# Source

This documentation was generated from [`CitationGroup.yaml`](https://github.com/stencila/stencila/blob/main/schema/CitationGroup.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
