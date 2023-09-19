# Heading

**A heading.**

Analogues of `Heading` in other schemas include:
  - HTML [`<h1>` to `<h6>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h1)
  - JATS XML [`<title>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/title.html)
  - Pandoc [`Header`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L233)


**`@id`**: `stencila:Heading`

## Properties

The `Heading` type has these properties:

| Name    | `@id`                                | Type                                                                                              | Description                  | Inherited from                                                                                     |
| ------- | ------------------------------------ | ------------------------------------------------------------------------------------------------- | ---------------------------- | -------------------------------------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The identifier for this item | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)   |
| depth   | `stencila:depth`                     | [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md) | The depth of the heading.    | [`Heading`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/heading.md) |
| content | `stencila:content`                   | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)* | Content of the heading.      | [`Heading`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/heading.md) |

## Related

The `Heading` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Heading` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                        | Encoding         | Decoding     | Status                 | Notes                          |
| --------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ------------------------------ |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)         | 🟢 No loss        |              | 🚧 Under development    | Encoded using special function |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)         | 🟢 No loss        |              | 🚧 Under development    |                                |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md) | 🟢 No loss        |              | 🚧 Under development    | Encoded using special function |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)   | ⚠️ High loss     |              | ⚠️ Alpha               |                                |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)       | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)         | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)       | 🔷 Low loss       |              | 🟢 Stable               |                                |

## Bindings

The `Heading` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Heading.jsonld)
- [JSON Schema](https://stencila.dev/Heading.schema.json)
- Python class [`Heading`](https://github.com/stencila/stencila/blob/main/python/stencila/types/heading.py)
- Rust struct [`Heading`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/heading.rs)
- TypeScript class [`Heading`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Heading.ts)

## Source

This documentation was generated from [`Heading.yaml`](https://github.com/stencila/stencila/blob/main/schema/Heading.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).