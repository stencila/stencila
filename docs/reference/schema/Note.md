# Note

**Additional content which is not part of the main content of a document.**

A note is usually associated with a word or paragraph using a number or other symbol. It can be displayed as a footnote, endnote, or side note, or in interactive elements. For analogues, see - [JATS `<fn>`](https://jats.nlm.nih.gov/publishing/tag-library/1.2/element/fn.html) - [Pandoc footnotes](https://pandoc.org/MANUAL.html#footnotes) - [HTML `<aside>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside)

This schema type is marked as **unstable** ⚠️ and is subject to change.

## Properties

| Name        | `@id`                                                         | Type                                     | Description                                                         | Inherited from      |
| ----------- | ------------------------------------------------------------- | ---------------------------------------- | ------------------------------------------------------------------- | ------------------- |
| **content** | [stencila:content](https://schema.stenci.la/content.jsonld)   | Array of [BlockContent](BlockContent.md) | Content of the note, usually a paragraph. See note [1](#notes).     | [Note](Note.md)     |
| id          | [schema:id](https://schema.org/id)                            | string                                   | The identifier for this item.                                       | [Entity](Entity.md) |
| meta        | [stencila:meta](https://schema.stenci.la/meta.jsonld)         | object                                   | Metadata associated with this item.                                 | [Entity](Entity.md) |
| noteType    | [stencila:noteType](https://schema.stenci.la/noteType.jsonld) | 'Footnote', 'Endnote', 'Sidenote'        | Determines where the note content is displayed within the document. | [Note](Note.md)     |

## Notes

1. **content** : Most notes will have a single paragraph but could have multiple paragraphs, tables and even figures.

## Related

- Parent: [Entity](Entity.md)
- Descendants: None

## Available as

- [JSON-LD](https://schema.stenci.la/Note.jsonld)
- [JSON Schema](https://schema.stenci.la/v1/Note.schema.json)
- Python [`class Note`](https://stencila.github.io/schema/python/docs/types.html#schema.types.Note)
- TypeScript [`interface Note`](https://stencila.github.io/schema/ts/docs/interfaces/note.html)
- R [`class Note`](https://cran.r-project.org/web/packages/stencilaschema/stencilaschema.pdf)
- Rust [`struct Note`](https://docs.rs/stencila-schema/latest/stencila_schema/struct.Note.html)

## Source

This documentation was generated from [Note.schema.yaml](https://github.com/stencila/stencila/blob/master/schema/schema/Note.schema.yaml).
