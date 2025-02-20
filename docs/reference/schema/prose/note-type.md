---
title: Note Type
description: The type of a `Note` which determines where the note content is displayed within the document.
config:
  publish:
    ghost:
      type: page
      slug: note-type
      state: publish
      tags:
      - '#schema'
      - '#doc'
      - Prose
---

## Members

The `NoteType` type has these members:

- `Footnote`
- `Endnote`
- `Sidenote`

## Bindings

The `NoteType` type is represented in:

- [JSON-LD](https://stencila.org/NoteType.jsonld)
- [JSON Schema](https://stencila.org/NoteType.schema.json)
- Python type [`NoteType`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/note_type.py)
- Rust type [`NoteType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/note_type.rs)
- TypeScript type [`NoteType`](https://github.com/stencila/stencila/blob/main/ts/src/types/NoteType.ts)

## Source

This documentation was generated from [`NoteType.yaml`](https://github.com/stencila/stencila/blob/main/schema/NoteType.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
