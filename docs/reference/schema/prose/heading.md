---
title:
- type: Text
  value: Heading
---

# Heading

**A heading.**

Analogues of `Heading` in other schemas include:
  - HTML [`<h1>` to `<h6>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h1)
  - JATS XML [`<title>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/title.html)
  - Pandoc [`Header`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L233)


**`@id`**: `stencila:Heading`

## Properties

The `Heading` type has these properties:

| Name    | `@id`                                | Type                                                                 | Description                  | Inherited from                                                        |
| ------- | ------------------------------------ | -------------------------------------------------------------------- | ---------------------------- | --------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | The identifier for this item | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)   |
| depth   | `stencila:depth`                     | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer) | The depth of the heading.    | [`Heading`](https://stencila.dev/docs/reference/schema/prose/heading) |
| content | `stencila:content`                   | [`Inline`](https://stencila.dev/docs/reference/schema/prose/inline)* | Content of the heading.      | [`Heading`](https://stencila.dev/docs/reference/schema/prose/heading) |

## Related

The `Heading` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `Heading` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                          |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ------------------------------ |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded using special function |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |                                |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟢 No loss      |              | 🚧 Under development    | Encoded using special function |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                |

## Bindings

The `Heading` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Heading.jsonld)
- [JSON Schema](https://stencila.dev/Heading.schema.json)
- Python class [`Heading`](https://github.com/stencila/stencila/blob/main/python/stencila/types/heading.py)
- Rust struct [`Heading`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/heading.rs)
- TypeScript class [`Heading`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Heading.ts)

## Source

This documentation was generated from [`Heading.yaml`](https://github.com/stencila/stencila/blob/main/schema/Heading.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).