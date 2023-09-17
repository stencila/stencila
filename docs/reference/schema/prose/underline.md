---
title:
- type: Text
  value: Underline
---

# Underline

**Inline text that is underlined.**

Analogues of `Underline` in other schema include:
- JATS XML [`<underline>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/underline.html)
- Pandoc [`Underline`](https://github.com/jgm/pandoc-types/blob/master/src/Text/Pandoc/Definition.hs)


**`@id`**: `stencila:Underline`

## Properties

The `Underline` type has these properties:

| Name    | `@id`                                | Type                                                                 | Description                  | Inherited from                                                      |
| ------- | ------------------------------------ | -------------------------------------------------------------------- | ---------------------------- | ------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | The identifier for this item | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| content | `stencila:content`                   | [`Inline`](https://stencila.dev/docs/reference/schema/prose/inline)* | The content that is marked.  | [`Mark`](https://stencila.dev/docs/reference/schema/prose/mark)     |

## Related

The `Underline` type is related to these types:

- Parents: [`Mark`](https://stencila.dev/docs/reference/schema/prose/mark)
- Children: none

## Formats

The `Underline` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                             |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ------------------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag `<u>`                              |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟢 No loss      |              | 🚧 Under development    | Encoded using template `[{content}]{{underline}}` |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                                   |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                   |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                   |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                   |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                                   |

## Bindings

The `Underline` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Underline.jsonld)
- [JSON Schema](https://stencila.dev/Underline.schema.json)
- Python class [`Underline`](https://github.com/stencila/stencila/blob/main/python/stencila/types/underline.py)
- Rust struct [`Underline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/underline.rs)
- TypeScript class [`Underline`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Underline.ts)

## Source

This documentation was generated from [`Underline.yaml`](https://github.com/stencila/stencila/blob/main/schema/Underline.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).