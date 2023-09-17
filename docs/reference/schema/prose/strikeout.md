---
title:
- type: Text
  value: Strikeout
---

# Strikeout

**Content that is marked as struck out**

Analogues of `Strikeout` in other schema include:
  - HTML [`<del>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del)
  - JATS XML [`<strike>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/strike.html)
  - MDAST [`Delete`](https://github.com/syntax-tree/mdast#delete)
  - Pandoc [`Strikeout`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L258)
Supersedes the `Delete` inline content type (the name "Strikeout" is less ambiguous than "Delete").


**`@id`**: `stencila:Strikeout`

## Properties

The `Strikeout` type has these properties:

| Name    | `@id`                                | Type                                                                 | Description                  | Inherited from                                                      |
| ------- | ------------------------------------ | -------------------------------------------------------------------- | ---------------------------- | ------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | The identifier for this item | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| content | `stencila:content`                   | [`Inline`](https://stencila.dev/docs/reference/schema/prose/inline)* | The content that is marked.  | [`Mark`](https://stencila.dev/docs/reference/schema/prose/mark)     |

## Related

The `Strikeout` type is related to these types:

- Parents: [`Mark`](https://stencila.dev/docs/reference/schema/prose/mark)
- Children: none

## Formats

The `Strikeout` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                  |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | -------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag `<s>`                   |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    | Encoded using template `~~{content}~~` |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                        |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                        |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                        |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                        |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                        |

## Bindings

The `Strikeout` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Strikeout.jsonld)
- [JSON Schema](https://stencila.dev/Strikeout.schema.json)
- Python class [`Strikeout`](https://github.com/stencila/stencila/blob/main/python/stencila/types/strikeout.py)
- Rust struct [`Strikeout`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/strikeout.rs)
- TypeScript class [`Strikeout`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Strikeout.ts)

## Source

This documentation was generated from [`Strikeout.yaml`](https://github.com/stencila/stencila/blob/main/schema/Strikeout.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).