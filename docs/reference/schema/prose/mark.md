# Mark

**Abstract base class for nodes that mark some other inline content in some way (e.g. as being emphasised, or quoted).**

**`@id`**: `stencila:Mark`

## Properties

The `Mark` type has these properties:

| Name      | Aliases | `@id`                                | Type                                                                                              | Description                   | Inherited from                                                                                   |
| --------- | ------- | ------------------------------------ | ------------------------------------------------------------------------------------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`      | -       | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The identifier for this item. | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `content` | -       | `stencila:content`                   | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)* | The content that is marked.   | -                                                                                                |

## Related

The `Mark` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: [`Annotation`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/annotation.md), [`Emphasis`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/emphasis.md), [`QuoteInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/quote-inline.md), [`Strikeout`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/strikeout.md), [`Strong`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/strong.md), [`Subscript`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/subscript.md), [`Superscript`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/superscript.md), [`Underline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/underline.md)

## Bindings

The `Mark` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Mark.jsonld)
- [JSON Schema](https://stencila.org/Mark.schema.json)
- Python class [`Mark`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/mark.py)
- Rust struct [`Mark`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/mark.rs)
- TypeScript class [`Mark`](https://github.com/stencila/stencila/blob/main/ts/src/types/Mark.ts)

## Source

This documentation was generated from [`Mark.yaml`](https://github.com/stencila/stencila/blob/main/schema/Mark.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
