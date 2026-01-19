---
title: Note
description: Additional content which is not part of the main content of a document.
---

A note is usually associated with a word or paragraph using a number or other symbol. 
It can be displayed as a footnote, endnote, or side note, or in interactive elements.
For analogues, see 
- [JATS `<fn>`](https://jats.nlm.nih.gov/publishing/tag-library/1.2/element/fn.html)
- [Pandoc footnotes](https://pandoc.org/MANUAL.html#footnotes)
- [HTML `<aside>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside)


# Properties

The `Note` type has these properties:

| Name       | Description                                                         | Type                         | Inherited from          | `JSON-LD @id`                        | Aliases                  |
| ---------- | ------------------------------------------------------------------- | ---------------------------- | ----------------------- | ------------------------------------ | ------------------------ |
| `id`       | The identifier for this item.                                       | [`String`](./string.md)      | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id) | -                        |
| `noteType` | Determines where the note content is displayed within the document. | [`NoteType`](./note-type.md) | -                       | `stencila:noteType`                  | `note-type`, `note_type` |
| `content`  | Content of the note, usually a paragraph.                           | [`Block`](./block.md)*       | -                       | `stencila:content`                   | -                        |

# Related

The `Note` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `Note` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                        | Notes |
| ------------------------------------------------ | ------------ | ------------ | ---------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |                                                                                                |
| [JATS](../formats/jats.md)                       | 🟢 No loss    | 🟢 No loss    | Encoded as [`<fn>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/fn.html) |
| [Markdown](../formats/md.md)                     | 🔷 Low loss   | 🔷 Low loss   | Encoded using implemented function                                                             |
| [Stencila Markdown](../formats/smd.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [Quarto Markdown](../formats/qmd.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [MyST Markdown](../formats/myst.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [LLM Markdown](../formats/llmd.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                                |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                                |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                                |
| [CSV](../formats/csv.md)                         |              |              |                                                                                                |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                                |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                                |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                                |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                                |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                                |
| [Directory](../formats/directory.md)             |              |              |                                                                                                |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                                |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                                |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                                |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                                |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                                |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                                |

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
