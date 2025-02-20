---
title: Mark
description: Abstract base class for nodes that mark some other inline content in some way (e.g. as being emphasised, or quoted).
config:
  publish:
    ghost:
      type: page
      slug: mark
      state: publish
      tags:
      - '#schema'
      - '#doc'
      - Prose
---

## Properties

The `Mark` type has these properties:

| Name      | Description                   | Type                                                                | Inherited from                                                     | `JSON-LD @id`                        | Aliases |
| --------- | ----------------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------ | ------- |
| `id`      | The identifier for this item. | [`String`](https://stencila.ghost.io/docs/reference/schema/string)  | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -       |
| `content` | The content that is marked.   | [`Inline`](https://stencila.ghost.io/docs/reference/schema/inline)* | -                                                                  | `stencila:content`                   | -       |

## Related

The `Mark` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: [`Annotation`](https://stencila.ghost.io/docs/reference/schema/annotation), [`Emphasis`](https://stencila.ghost.io/docs/reference/schema/emphasis), [`QuoteInline`](https://stencila.ghost.io/docs/reference/schema/quote-inline), [`Strikeout`](https://stencila.ghost.io/docs/reference/schema/strikeout), [`Strong`](https://stencila.ghost.io/docs/reference/schema/strong), [`Subscript`](https://stencila.ghost.io/docs/reference/schema/subscript), [`Superscript`](https://stencila.ghost.io/docs/reference/schema/superscript), [`Underline`](https://stencila.ghost.io/docs/reference/schema/underline)

## Bindings

The `Mark` type is represented in:

- [JSON-LD](https://stencila.org/Mark.jsonld)
- [JSON Schema](https://stencila.org/Mark.schema.json)
- Python class [`Mark`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/mark.py)
- Rust struct [`Mark`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/mark.rs)
- TypeScript class [`Mark`](https://github.com/stencila/stencila/blob/main/ts/src/types/Mark.ts)

## Source

This documentation was generated from [`Mark.yaml`](https://github.com/stencila/stencila/blob/main/schema/Mark.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
