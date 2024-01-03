# Suggestion Block

**Abstract base type for nodes that indicate a suggested change to block content.**

**`@id`**: `stencila:SuggestionBlock`

## Properties

The `SuggestionBlock` type has these properties:

| Name      | Aliases | `@id`                                | Type                                                                                            | Description                                                                   | Inherited from                                                                                   |
| --------- | ------- | ------------------------------------ | ----------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`      | -       | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item.                                                 | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `content` | -       | `stencila:content`                   | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)* | The content that is suggested to be inserted, modified, replaced, or deleted. | -                                                                                                |

## Related

The `SuggestionBlock` type is related to these types:

- Parents: [`Suggestion`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion.md)
- Children: [`DeleteBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/delete-block.md), [`InsertBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/insert-block.md), [`ModifyBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify-block.md), [`ReplaceBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/replace-block.md)

## Bindings

The `SuggestionBlock` type is represented in these bindings:

- [JSON-LD](https://stencila.org/SuggestionBlock.jsonld)
- [JSON Schema](https://stencila.org/SuggestionBlock.schema.json)
- Python class [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/suggestion_block.py)
- Rust struct [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion_block.rs)
- TypeScript class [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/SuggestionBlock.ts)

## Source

This documentation was generated from [`SuggestionBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/SuggestionBlock.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).