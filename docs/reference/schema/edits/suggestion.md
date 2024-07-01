# Suggestion

**Abstract base type for nodes that indicate a suggested change to content.**

**`@id`**: `stencila:Suggestion`

## Properties

The `Suggestion` type has these properties:

| Name                | Aliases                                    | `@id`                                        | Type                                                                                                                  | Description                                                                           | Inherited from                                                                                   |
| ------------------- | ------------------------------------------ | -------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`                | -                                          | [`schema:id`](https://schema.org/id)         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                       | The identifier for this item.                                                         | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `suggestionStatus`  | `suggestion-status`, `suggestion_status`   | `stencila:suggestionStatus`                  | [`SuggestionStatus`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-status.md) | The status of the suggestion including whether it is proposed, accepted, or rejected. | -                                                                                                |
| `authors`           | `author`                                   | [`schema:author`](https://schema.org/author) | [`Author`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author.md)*                     | The authors of the suggestion                                                         | -                                                                                                |
| `provenance`        | -                                          | `stencila:provenance`                        | [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/provenance-count.md)*  | A summary of the provenance of the content within the suggestion.                     | -                                                                                                |
| `executionDuration` | `execution-duration`, `execution_duration` | `stencila:executionDuration`                 | [`Duration`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)                   | Time taken to generate the suggestion.                                                | -                                                                                                |
| `executionEnded`    | `execution-ended`, `execution_ended`       | `stencila:executionEnded`                    | [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md)                 | The timestamp when the generation ended.                                              | -                                                                                                |
| `feedback`          | -                                          | `stencila:feedback`                          | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                       | Feedback on the suggestion                                                            | -                                                                                                |

## Related

The `Suggestion` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-block.md), [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-inline.md)

## Bindings

The `Suggestion` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Suggestion.jsonld)
- [JSON Schema](https://stencila.org/Suggestion.schema.json)
- Python class [`Suggestion`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/suggestion.py)
- Rust struct [`Suggestion`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion.rs)
- TypeScript class [`Suggestion`](https://github.com/stencila/stencila/blob/main/ts/src/types/Suggestion.ts)

## Source

This documentation was generated from [`Suggestion.yaml`](https://github.com/stencila/stencila/blob/main/schema/Suggestion.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
