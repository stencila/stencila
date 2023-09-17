---
title:
- type: Text
  value: Mark
---

# Mark

**Abstract base class for nodes that mark some other inline content
in some way (e.g. as being emphasised, or quoted).
**

**`@id`**: `stencila:Mark`

## Properties

The `Mark` type has these properties:

| Name    | `@id`                                | Type                                                                 | Description                  | Inherited from                                                      |
| ------- | ------------------------------------ | -------------------------------------------------------------------- | ---------------------------- | ------------------------------------------------------------------- |
| id      | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | The identifier for this item | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |
| content | `stencila:content`                   | [`Inline`](https://stencila.dev/docs/reference/schema/prose/inline)* | The content that is marked.  | [`Mark`](https://stencila.dev/docs/reference/schema/prose/mark)     |

## Related

The `Mark` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: [`Emphasis`](https://stencila.dev/docs/reference/schema/prose/emphasis), [`Quote`](https://stencila.dev/docs/reference/schema/prose/quote), [`Strikeout`](https://stencila.dev/docs/reference/schema/prose/strikeout), [`Strong`](https://stencila.dev/docs/reference/schema/prose/strong), [`Subscript`](https://stencila.dev/docs/reference/schema/prose/subscript), [`Superscript`](https://stencila.dev/docs/reference/schema/prose/superscript), [`Underline`](https://stencila.dev/docs/reference/schema/prose/underline)

## Bindings

The `Mark` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Mark.jsonld)
- [JSON Schema](https://stencila.dev/Mark.schema.json)
- Python class [`Mark`](https://github.com/stencila/stencila/blob/main/python/stencila/types/mark.py)
- Rust struct [`Mark`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/mark.rs)
- TypeScript class [`Mark`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Mark.ts)

## Source

This documentation was generated from [`Mark.yaml`](https://github.com/stencila/stencila/blob/main/schema/Mark.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).