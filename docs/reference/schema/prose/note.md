---
title:
- type: Text
  value: Note
---

# Note

**Additional content which is not part of the main content of a document.**

A note is usually associated with a word or paragraph using a number or other symbol. 
It can be displayed as a footnote, endnote, or side note, or in interactive elements.
For analogues, see 
- [JATS `<fn>`](https://jats.nlm.nih.gov/publishing/tag-library/1.2/element/fn.html)
- [Pandoc footnotes](https://pandoc.org/MANUAL.html#footnotes)
- [HTML `<aside>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside)


**`@id`**: `stencila:Note`

## Properties

The `Note` type has these properties:

| Name     | `@id`                                | Type                                                                     | Description                                                         | Inherited from                                                      |
| -------- | ------------------------------------ | ------------------------------------------------------------------------ | ------------------------------------------------------------------- | ------------------------------------------------------------------- |
| id       | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)       | The identifier for this item                                        | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| noteType | `stencila:noteType`                  | [`NoteType`](https://stencila.dev/docs/reference/schema/prose/note-type) | Determines where the note content is displayed within the document. | [`Note`](https://stencila.dev/docs/reference/schema/prose/note)     |
| content  | `stencila:content`                   | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)*       | Content of the note, usually a paragraph.                           | [`Note`](https://stencila.dev/docs/reference/schema/prose/note)     |

## Related

The `Note` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `Note` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `Note` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Note.jsonld)
- [JSON Schema](https://stencila.dev/Note.schema.json)
- Python class [`Note`](https://github.com/stencila/stencila/blob/main/python/stencila/types/note.py)
- Rust struct [`Note`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/note.rs)
- TypeScript class [`Note`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Note.ts)

## Source

This documentation was generated from [`Note.yaml`](https://github.com/stencila/stencila/blob/main/schema/Note.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).