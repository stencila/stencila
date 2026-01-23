---
title: Suggestion Inline
description: Abstract base type for nodes that indicate a suggested change to inline content.
---

# Properties

The `SuggestionInline` type has these properties:

| Name                | Description                                                                                     | Type                                         | Inherited from                  |
| ------------------- | ----------------------------------------------------------------------------------------------- | -------------------------------------------- | ------------------------------- |
| `id`                | The identifier for this item.                                                                   | [`String`](./string.md)                      | [`Entity`](./entity.md)         |
| `suggestionStatus`  | The status of the suggestion including whether it is the original, or is accepted, or rejected. | [`SuggestionStatus`](./suggestion-status.md) | [`Suggestion`](./suggestion.md) |
| `authors`           | The authors of the suggestion                                                                   | [`Author`](./author.md)*                     | [`Suggestion`](./suggestion.md) |
| `provenance`        | A summary of the provenance of the content within the suggestion.                               | [`ProvenanceCount`](./provenance-count.md)*  | [`Suggestion`](./suggestion.md) |
| `executionDuration` | Time taken to generate the suggestion.                                                          | [`Duration`](./duration.md)                  | [`Suggestion`](./suggestion.md) |
| `executionEnded`    | The timestamp when the generation ended.                                                        | [`Timestamp`](./timestamp.md)                | [`Suggestion`](./suggestion.md) |
| `feedback`          | Feedback on the suggestion                                                                      | [`String`](./string.md)                      | [`Suggestion`](./suggestion.md) |
| `content`           | The content that is suggested to be inserted, modified, replaced, or deleted.                   | [`Inline`](./inline.md)*                     | -                               |

# Related

The `SuggestionInline` type is related to these types:

- Parents: [`Suggestion`](./suggestion.md)
- Children: none

# Bindings

The `SuggestionInline` type is represented in:

- [JSON-LD](https://stencila.org/SuggestionInline.jsonld)
- [JSON Schema](https://stencila.org/SuggestionInline.schema.json)
- Python class [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion_inline.rs)
- TypeScript class [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/SuggestionInline.ts)

***

This documentation was generated from [`SuggestionInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/SuggestionInline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
