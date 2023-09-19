# Paragraph

**Paragraph**

Analogues of `Paragraph` in other schema include:
  - HTML [`<p>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p)
  - JATS XML [`<p>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/p.html)
  - MDAST [`Paragraph`](https://github.com/syntax-tree/mdast#Paragraph)
  - OpenDocument [`<text:p>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415138_253892949)
  - Pandoc [`Para`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L220)


**`@id`**: `stencila:Paragraph`

## Properties

The `Paragraph` type has these properties:

| Name    | `@id`                                | Type                                                                                              | Description                    | Inherited from                                                                                         |
| ------- | ------------------------------------ | ------------------------------------------------------------------------------------------------- | ------------------------------ | ------------------------------------------------------------------------------------------------------ |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The identifier for this item   | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)       |
| content | `stencila:content`                   | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)* | The contents of the paragraph. | [`Paragraph`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/paragraph.md) |

## Related

The `Paragraph` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Paragraph` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding     | Status                 | Notes                                                                                       |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ------------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🟢 No loss        |              | 🚧 Under development    | Encoded to tag [`<p>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p)         |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         | 🟢 No loss        |              | 🚧 Under development    | Encoded to tag [`<p>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/p) |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | 🟢 No loss        |              | 🚧 Under development    | Encoded using template `{content}\n\n`                                                      |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                             |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                             |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                             |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                             |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |              | 🟢 Stable               |                                                                                             |

## Bindings

The `Paragraph` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Paragraph.jsonld)
- [JSON Schema](https://stencila.dev/Paragraph.schema.json)
- Python class [`Paragraph`](https://github.com/stencila/stencila/blob/main/python/stencila/types/paragraph.py)
- Rust struct [`Paragraph`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/paragraph.rs)
- TypeScript class [`Paragraph`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Paragraph.ts)

## Source

This documentation was generated from [`Paragraph.yaml`](https://github.com/stencila/stencila/blob/main/schema/Paragraph.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).