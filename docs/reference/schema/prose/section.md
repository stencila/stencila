# Section

**A section of a document**

**`@id`**: `stencila:Section`

## Properties

The `Section` type has these properties:

| Name    | `@id`                                | Type                                                                                            | Description                    | Inherited from                                                                                     |
| ------- | ------------------------------------ | ----------------------------------------------------------------------------------------------- | ------------------------------ | -------------------------------------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item   | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)   |
| content | `stencila:content`                   | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)* | The content within the section | [`Section`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/section.md) |

## Related

The `Section` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Section` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding     | Status                 | Notes                                                                                                |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ---------------------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🟢 No loss        |              | 🚧 Under development    | Encoded to tag [`<section>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/section)      |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         | 🟢 No loss        | 🟢 No loss    | 🚧 Under development    | Encoded to tag [`<sec>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/sec.html) |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | 🟢 No loss        |              | 🚧 Under development    | Encoded using special function                                                                       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |              | ⚠️ Alpha               |                                                                                                      |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                      |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                      |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                                                                                      |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |              | 🟢 Stable               |                                                                                                      |

## Bindings

The `Section` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Section.jsonld)
- [JSON Schema](https://stencila.dev/Section.schema.json)
- Python class [`Section`](https://github.com/stencila/stencila/blob/main/python/stencila/types/section.py)
- Rust struct [`Section`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/section.rs)
- TypeScript class [`Section`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Section.ts)

## Source

This documentation was generated from [`Section.yaml`](https://github.com/stencila/stencila/blob/main/schema/Section.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).