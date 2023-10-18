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

| Name     | `@id`                                | Type                                                                                                  | Description                                                         | Inherited from                                                                                   |
| -------- | ------------------------------------ | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| id       | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)       | The identifier for this item.                                       | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| noteType | `stencila:noteType`                  | [`NoteType`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/note-type.md) | Determines where the note content is displayed within the document. | -                                                                                                |
| content  | `stencila:content`                   | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*       | Content of the note, usually a paragraph.                           | -                                                                                                |

## Related

The `Note` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Note` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding     | Status                 | Notes                                                                                              |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | -------------------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🔷 Low loss       |              | 🚧 Under development    |                                                                                                    |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         | 🟢 No loss        | 🟢 No loss    | 🚧 Under development    | Encoded to tag [`<fn>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/fn.html) |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | ⚠️ High loss     |              | 🚧 Under development    |                                                                                                    |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |              | 🟢 Stable               |                                                                                                    |

## Bindings

The `Note` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Note.jsonld)
- [JSON Schema](https://stencila.dev/Note.schema.json)
- Python class [`Note`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/note.py)
- Rust struct [`Note`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/note.rs)
- TypeScript class [`Note`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Note.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `Note` type are generated using the following strategies for each complexity level (see the [`proptest` book](https://proptest-rs.github.io/proptest/) for an explanation of the Rust strategy expressions). Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                  | Strategy                                                                                                        |
| --------- | ---------- | ------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------- |
| `content` | Min+       | Generate a single paragraph not a note (to avoid recursion). | `vec![Block::Paragraph(crate::Paragraph::new(vec![crate::Inline::Text(crate::Text::from("Note paragraph"))]))]` |

## Source

This documentation was generated from [`Note.yaml`](https://github.com/stencila/stencila/blob/main/schema/Note.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).