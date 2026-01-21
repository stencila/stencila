---
title: Mark
description: Abstract base class for nodes that mark some other inline content in some way (e.g. as being emphasised, or quoted).
---

# Properties

The `Mark` type has these properties:

| Name      | Description                   | Type                     | Inherited from          |
| --------- | ----------------------------- | ------------------------ | ----------------------- |
| `id`      | The identifier for this item. | [`String`](./string.md)  | [`Entity`](./entity.md) |
| `content` | The content that is marked.   | [`Inline`](./inline.md)* | -                       |

# Related

The `Mark` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: [`Annotation`](./annotation.md), [`Emphasis`](./emphasis.md), [`QuoteInline`](./quote-inline.md), [`Strikeout`](./strikeout.md), [`Strong`](./strong.md), [`Subscript`](./subscript.md), [`Superscript`](./superscript.md), [`Underline`](./underline.md)

# Bindings

The `Mark` type is represented in:

- [JSON-LD](https://stencila.org/Mark.jsonld)
- [JSON Schema](https://stencila.org/Mark.schema.json)
- Python class [`Mark`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/mark.py)
- Rust struct [`Mark`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/mark.rs)
- TypeScript class [`Mark`](https://github.com/stencila/stencila/blob/main/ts/src/types/Mark.ts)

# Source

This documentation was generated from [`Mark.yaml`](https://github.com/stencila/stencila/blob/main/schema/Mark.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
