---
title: Note
description: A note associated with document content.
---

This is a type used in Stencila Schema for notes attached to document content.

It exists to represent note content structurally so it can be rendered as a
footnote, endnote, sidenote, or similar document note, depending on the target
format and note type. This preserves note semantics across transformations
rather than reducing them to plain text markers.

Key properties include `noteType` and `content`.


# Analogues

The following external types, elements, or nodes are similar to a `Note`:

- HTML [`<aside>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside): Approximate HTML analogue for note content, though HTML has no dedicated footnote or endnote element and placement semantics are not built in.
- JATS [`<fn>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/fn.html): Closest JATS analogue for document notes, particularly footnotes.
- Pandoc [`Note`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Note): Closest Pandoc analogue for inline-referenced notes; Stencila additionally records note placement semantics explicitly via `noteType`.
- MDAST [`FootnoteDefinition`](https://github.com/syntax-tree/mdast#footnotedefinition): Close MDAST analogue for defined footnote content, although MDAST models note references and definitions separately and does not cover sidenotes directly.

# Properties

The `Note` type has these properties:

| Name       | Description                                                         | Type                         | Inherited from          |
| ---------- | ------------------------------------------------------------------- | ---------------------------- | ----------------------- |
| `noteType` | Determines where the note content is displayed within the document. | [`NoteType`](./note-type.md) | -                       |
| `content`  | Content of the note, usually a paragraph.                           | [`Block`](./block.md)*       | -                       |
| `id`       | The identifier for this item.                                       | [`String`](./string.md)      | [`Entity`](./entity.md) |

# Related

The `Note` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Note` type is represented in:

- [JSON-LD](https://stencila.org/Note.jsonld)
- [JSON Schema](https://stencila.org/Note.schema.json)
- Python class [`Note`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
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

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`Note.yaml`](https://github.com/stencila/stencila/blob/main/schema/Note.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
