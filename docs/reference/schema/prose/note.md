---
title: Note
description: Additional content which is not part of the main content of a document.
config:
  publish:
    ghost:
      type: post
      slug: note
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

A note is usually associated with a word or paragraph using a number or other symbol. 
It can be displayed as a footnote, endnote, or side note, or in interactive elements.
For analogues, see 
- [JATS `<fn>`](https://jats.nlm.nih.gov/publishing/tag-library/1.2/element/fn.html)
- [Pandoc footnotes](https://pandoc.org/MANUAL.html#footnotes)
- [HTML `<aside>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside)


# Properties

The `Note` type has these properties:

| Name       | Description                                                         | Type                                                                    | Inherited from                                                     | `JSON-LD @id`                        | Aliases                  |
| ---------- | ------------------------------------------------------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------ | ------------------------ |
| `id`       | The identifier for this item.                                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)      | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -                        |
| `noteType` | Determines where the note content is displayed within the document. | [`NoteType`](https://stencila.ghost.io/docs/reference/schema/note-type) | -                                                                  | `stencila:noteType`                  | `note-type`, `note_type` |
| `content`  | Content of the note, usually a paragraph.                           | [`Block`](https://stencila.ghost.io/docs/reference/schema/block)*       | -                                                                  | `stencila:content`                   | -                        |

# Related

The `Note` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `Note` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                                                                                        | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | ---------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                                                                                |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |                                                                                                |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       | 🟢 No loss    | 🟢 No loss    | Encoded as [`<fn>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/fn.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | 🔷 Low loss   | 🔷 Low loss   | Encoded using implemented function                                                             |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                                                                                |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |                                                                                                |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                                                                                |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |                                                                                                |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                                                                                |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |                                                                                                |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                                                                                |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                                                                                |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                                                                                |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                                                                                |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |                                                                                                |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                                                                                |

# Bindings

The `Note` type is represented in:

- [JSON-LD](https://stencila.org/Note.jsonld)
- [JSON Schema](https://stencila.org/Note.schema.json)
- Python class [`Note`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/note.py)
- Rust struct [`Note`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/note.rs)
- TypeScript class [`Note`](https://github.com/stencila/stencila/blob/main/ts/src/types/Note.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Note` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property   | Complexity | Description                                                      | Strategy                         |
| ---------- | ---------- | ---------------------------------------------------------------- | -------------------------------- |
| `noteType` | Min+       | Fixed footnote type.                                             | `NoteType::Footnote`             |
|            | High+      | Generate an arbitrary note type.                                 | `NoteType::arbitrary()`          |
| `content`  | Min+       | Generate a single paragraph (with no `Note` to avoid recursion). | `vec![p([t("Note paragraph")])]` |

# Source

This documentation was generated from [`Note.yaml`](https://github.com/stencila/stencila/blob/main/schema/Note.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
