---
title:
- type: Text
  value: Quote
---

# Quote

**Inline, quoted content.**

**`@id`**: `stencila:Quote`

## Properties

The `Quote` type has these properties:

| Name    | `@id`                                | Type                                                                                                                                 | Description                  | Inherited from                                                      |
| ------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------- | ------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                   | The identifier for this item | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| content | `stencila:content`                   | [`Inline`](https://stencila.dev/docs/reference/schema/prose/inline)*                                                                 | The content that is marked.  | [`Mark`](https://stencila.dev/docs/reference/schema/prose/mark)     |
| cite    | `stencila:cite`                      | [`Cite`](https://stencila.dev/docs/reference/schema/prose/cite) \| [`String`](https://stencila.dev/docs/reference/schema/data/string) | The source of the quote.     | [`Quote`](https://stencila.dev/docs/reference/schema/prose/quote)   |

## Related

The `Quote` type is related to these types:

- Parents: [`Mark`](https://stencila.dev/docs/reference/schema/prose/mark)
- Children: none

## Formats

The `Quote` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                                                               |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----------------------------------------------------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<q>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/q) |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |                                                                                     |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    | Encoded using template `<q>{content}</q>`                                           |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                                                                     |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                     |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                                                                     |

## Bindings

The `Quote` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Quote.jsonld)
- [JSON Schema](https://stencila.dev/Quote.schema.json)
- Python class [`Quote`](https://github.com/stencila/stencila/blob/main/python/stencila/types/quote.py)
- Rust struct [`Quote`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/quote.rs)
- TypeScript class [`Quote`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Quote.ts)

## Source

This documentation was generated from [`Quote.yaml`](https://github.com/stencila/stencila/blob/main/schema/Quote.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).