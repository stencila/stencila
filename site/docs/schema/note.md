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

| Name       | Description                                                         | Type                         | Inherited from          |
| ---------- | ------------------------------------------------------------------- | ---------------------------- | ----------------------- |
| `id`       | The identifier for this item.                                       | [`String`](./string.md)      | [`Entity`](./entity.md) |
| `noteType` | Determines where the note content is displayed within the document. | [`NoteType`](./note-type.md) | -                       |
| `content`  | Content of the note, usually a paragraph.                           | [`Block`](./block.md)*       | -                       |

# Related

The `Note` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Note` type is represented in:

- [JSON-LD](https://stencila.org/Note.jsonld)
- [JSON Schema](https://stencila.org/Note.schema.json)
- Python class [`Note`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/note.py)
- Rust struct [`Note`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/note.rs)
- TypeScript class [`Note`](https://github.com/stencila/stencila/blob/main/ts/src/types/Note.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Note` type are generated using the following strategies.

::: table

| Property   | Complexity | Description                                                      | Strategy                         |
| ---------- | ---------- | ---------------------------------------------------------------- | -------------------------------- |
| `noteType` | Min+       | Fixed footnote type.                                             | `NoteType::Footnote`             |
|            | High+      | Generate an arbitrary note type.                                 | `NoteType::arbitrary()`          |
| `content`  | Min+       | Generate a single paragraph (with no `Note` to avoid recursion). | `vec![p([t("Note paragraph")])]` |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`Note.yaml`](https://github.com/stencila/stencila/blob/main/schema/Note.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
