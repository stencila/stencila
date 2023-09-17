---
title:
- type: Text
  value: Emphasis
---

# Emphasis

**Emphasized content.**

Analogues of `Delete` in other schema include:
  - HTML [`<em>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em)
  - JATS XML [`<italic>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/italic.html)
  - MDAST [`Emphasis`](https://github.com/syntax-tree/mdast#emphasis)
  - Pandoc [`Emph`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L256)


**`@id`**: `stencila:Emphasis`

## Properties

The `Emphasis` type has these properties:

| Name    | `@id`                                | Type                                                                 | Description                  | Inherited from                                                      |
| ------- | ------------------------------------ | -------------------------------------------------------------------- | ---------------------------- | ------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | The identifier for this item | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| content | `stencila:content`                   | [`Inline`](https://stencila.dev/docs/reference/schema/prose/inline)* | The content that is marked.  | [`Mark`](https://stencila.dev/docs/reference/schema/prose/mark)     |

## Related

The `Emphasis` type is related to these types:

- Parents: [`Mark`](https://stencila.dev/docs/reference/schema/prose/mark)
- Children: none

## Formats

The `Emphasis` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ------------------------------------ |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag `<em>`                |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟢 No loss      |              | 🚧 Under development    | Encoded using template `_{content}_` |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                      |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                      |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                      |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                      |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                      |

## Bindings

The `Emphasis` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Emphasis.jsonld)
- [JSON Schema](https://stencila.dev/Emphasis.schema.json)
- Python class [`Emphasis`](https://github.com/stencila/stencila/blob/main/python/stencila/types/emphasis.py)
- Rust struct [`Emphasis`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/emphasis.rs)
- TypeScript class [`Emphasis`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Emphasis.ts)

## Source

This documentation was generated from [`Emphasis.yaml`](https://github.com/stencila/stencila/blob/main/schema/Emphasis.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).