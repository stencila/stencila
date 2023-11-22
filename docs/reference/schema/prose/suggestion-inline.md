# Suggestion Inline

**Abstract base type for nodes that indicate a suggested change to inline content.**

**`@id`**: `stencila:SuggestionInline`

## Properties

The `SuggestionInline` type has these properties:

| Name      | Aliases | `@id`                                | Type                                                                                              | Description                                              | Inherited from                                                                                   |
| --------- | ------- | ------------------------------------ | ------------------------------------------------------------------------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`      | -       | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The identifier for this item.                            | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `content` | -       | `stencila:content`                   | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)* | The content that is suggested to be inserted or deleted. | -                                                                                                |

## Related

The `SuggestionInline` type is related to these types:

- Parents: [`Suggestion`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/suggestion.md)
- Children: [`DeleteInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/delete-inline.md), [`InsertInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/insert-inline.md)

## Bindings

The `SuggestionInline` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/SuggestionInline.jsonld)
- [JSON Schema](https://stencila.dev/SuggestionInline.schema.json)
- Python class [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/suggestion_inline.py)
- Rust struct [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion_inline.rs)
- TypeScript class [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/typescript/src/types/SuggestionInline.ts)

## Source

This documentation was generated from [`SuggestionInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/SuggestionInline.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).