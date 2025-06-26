---
title: Cord
description: A value comprised of a sequence of characters.
config:
  publish:
    ghost:
      type: post
      slug: cord
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Data
---

This type exists to differentiate between between a plain string of characters
(which is modified by complete replacement) and a sequence of characters stored and
synchronized as a CRDT (which is is modified by insertions and deletions).

Its use includes the `value` property of the `Text` type and the `code`
property of `CodeExecutable` nodes.


# Formats

The `Cord` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding   | Support | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ---------- | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |            |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |            |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       | 🟢 No loss    | 🟢 No loss  |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | 🟢 No loss    | 🟢 No loss  |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | 🟢 No loss    | 🟢 No loss  |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | 🟢 No loss    | 🟢 No loss  |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | 🟢 No loss    | 🟢 No loss  |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | 🟢 No loss    | 🟢 No loss  |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss |         |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss |            |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |            |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss |         |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |            |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss  |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss  |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss  |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss  |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss  |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss |         |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |            |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |            |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |            |         |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss |         |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |            |         |

# Bindings

The `Cord` type is represented in:

- [JSON-LD](https://stencila.org/Cord.jsonld)
- [JSON Schema](https://stencila.org/Cord.schema.json)
- Python type [`Cord`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/cord.py)
- Rust type [`Cord`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/cord.rs)
- TypeScript type [`Cord`](https://github.com/stencila/stencila/blob/main/ts/src/types/Cord.ts)

# Source

This documentation was generated from [`Cord.yaml`](https://github.com/stencila/stencila/blob/main/schema/Cord.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
