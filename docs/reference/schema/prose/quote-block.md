---
title:
- type: Text
  value: QuoteBlock
---

# Quote Block

**A section quoted from somewhere else.
**

**`@id`**: `stencila:QuoteBlock`

## Properties

The `QuoteBlock` type has these properties:

| Name    | `@id`                                | Type                                                                                                                                 | Description                  | Inherited from                                                               |
| ------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------- | ---------------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                   | The identifier for this item | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)          |
| cite    | `stencila:cite`                      | [`Cite`](https://stencila.dev/docs/reference/schema/prose/cite) \| [`String`](https://stencila.dev/docs/reference/schema/data/string) | The source of the quote.     | [`QuoteBlock`](https://stencila.dev/docs/reference/schema/prose/quote-block) |
| content | `stencila:content`                   | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)*                                                                   | The content of the quote.    | [`QuoteBlock`](https://stencila.dev/docs/reference/schema/prose/quote-block) |

## Related

The `QuoteBlock` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `QuoteBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `QuoteBlock` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/QuoteBlock.jsonld)
- [JSON Schema](https://stencila.dev/QuoteBlock.schema.json)
- Python class [`QuoteBlock`](https://github.com/stencila/stencila/blob/main/python/stencila/types/quote_block.py)
- Rust struct [`QuoteBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/quote_block.rs)
- TypeScript class [`QuoteBlock`](https://github.com/stencila/stencila/blob/main/typescript/src/types/QuoteBlock.ts)

## Source

This documentation was generated from [`QuoteBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/QuoteBlock.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).