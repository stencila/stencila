# Suggestion

**Abstract base type for nodes that indicate a suggested change to content.**

Note that at present this type does not have a `suggester` property to identify the
actor (`Person`, `Organization`, or `SoftwareApplication`) which suggested the change.
That is because, the creator of a `Suggestion` node is inherently the suggester and 
will be identifiable from the node's id.

The suggester will normally be amongst the `authors`, `contributors`, or `maintainers`
of the `CreativeWork`.


**`@id`**: `stencila:Suggestion`

## Properties

The `Suggestion` type has these properties:

| Name | Aliases | `@id`                                | Type                                                                                            | Description                   | Inherited from                                                                                   |
| ---- | ------- | ------------------------------------ | ----------------------------------------------------------------------------------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------ |
| `id` | -       | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item. | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |

## Related

The `Suggestion` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/suggestion-block.md), [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/suggestion-inline.md)

## Bindings

The `Suggestion` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Suggestion.jsonld)
- [JSON Schema](https://stencila.dev/Suggestion.schema.json)
- Python class [`Suggestion`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/suggestion.py)
- Rust struct [`Suggestion`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion.rs)
- TypeScript class [`Suggestion`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Suggestion.ts)

## Source

This documentation was generated from [`Suggestion.yaml`](https://github.com/stencila/stencila/blob/main/schema/Suggestion.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).