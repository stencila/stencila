---
title: Note Type
description: A category of note placement.
---

This is an enumeration used in Stencila Schema for note placement and behavior.

It exists so notes can be rendered and transformed consistently across formats
while preserving their semantic role, such as footnotes or endnotes, rather
than only their presentation.

See [`Note.noteType`](./note.md#notetype) for the property that uses this
enumeration.


# Analogues

The following external types, elements, or nodes are similar to a `NoteType`:

- [JATS fn-type values](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/attribute/fn-type.html): Close JATS analogue for classifying note kinds, though Stencila focuses on placement-oriented note types such as footnote, endnote, and sidenote.

# Members

The `NoteType` type has these members:

| Member     | Description |
| ---------- | ----------- |
| `Footnote` | -           |
| `Endnote`  | -           |
| `Sidenote` | -           |

# Bindings

The `NoteType` type is represented in:

- [JSON-LD](https://stencila.org/NoteType.jsonld)
- [JSON Schema](https://stencila.org/NoteType.schema.json)
- Python type [`NoteType`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`NoteType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/note_type.rs)
- TypeScript type [`NoteType`](https://github.com/stencila/stencila/blob/main/ts/src/types/NoteType.ts)

***

This documentation was generated from [`NoteType.yaml`](https://github.com/stencila/stencila/blob/main/schema/NoteType.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
