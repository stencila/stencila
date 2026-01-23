---
title: Sentence
description: A sentence, usually within a `Paragraph`.
---

# Properties

The `Sentence` type has these properties:

| Name      | Description                   | Type                     | Inherited from          |
| --------- | ----------------------------- | ------------------------ | ----------------------- |
| `id`      | The identifier for this item. | [`String`](./string.md)  | [`Entity`](./entity.md) |
| `content` | The content of the sentence.  | [`Inline`](./inline.md)* | -                       |

# Related

The `Sentence` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Sentence` type is represented in:

- [JSON-LD](https://stencila.org/Sentence.jsonld)
- [JSON Schema](https://stencila.org/Sentence.schema.json)
- Python class [`Sentence`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Sentence`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/sentence.rs)
- TypeScript class [`Sentence`](https://github.com/stencila/stencila/blob/main/ts/src/types/Sentence.ts)

***

This documentation was generated from [`Sentence.yaml`](https://github.com/stencila/stencila/blob/main/schema/Sentence.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
