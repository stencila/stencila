# Superscript

**Superscripted content.**

**`@id`**: `stencila:Superscript`

## Properties

The `Superscript` type has these properties:

| Name    | `@id`                                | Type                                                                                              | Description                  | Inherited from                                                                                   |
| ------- | ------------------------------------ | ------------------------------------------------------------------------------------------------- | ---------------------------- | ------------------------------------------------------------------------------------------------ |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The identifier for this item | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| content | `stencila:content`                   | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)* | The content that is marked.  | [`Mark`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/mark.md)     |

## Related

The `Superscript` type is related to these types:

- Parents: [`Mark`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/mark.md)
- Children: none

## Formats

The `Superscript` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                            | Encoding       | Decoding     | Status                 | Notes                                                                                           |
| ------------------------------------------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----------------------------------------------------------------------------------------------- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/HTML.md)             | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<sup>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sup)         |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JATS.md)             | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<sup>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/sup) |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Markdown.md)     | 🟢 No loss      |              | 🚧 Under development    | Encoded using template `^{content}^`                                                            |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Plain text.md) | 🟥 High loss    |              | 🟥 Alpha                |                                                                                                 |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JSON.md)             | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                 |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JSON5.md)           | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                 |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/YAML.md)             | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                 |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Debug.md)           | 🔷 Low loss     |              | 🟢 Stable               |                                                                                                 |

## Bindings

The `Superscript` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Superscript.jsonld)
- [JSON Schema](https://stencila.dev/Superscript.schema.json)
- Python class [`Superscript`](https://github.com/stencila/stencila/blob/main/python/stencila/types/superscript.py)
- Rust struct [`Superscript`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/superscript.rs)
- TypeScript class [`Superscript`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Superscript.ts)

## Source

This documentation was generated from [`Superscript.yaml`](https://github.com/stencila/stencila/blob/main/schema/Superscript.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).