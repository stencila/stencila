---
title: Suggestion
description: Abstract base type for nodes that indicate a suggested change to content.
---

# Properties

The `Suggestion` type has these properties:

| Name                | Description                                                                                     | Type                                         | Inherited from          |
| ------------------- | ----------------------------------------------------------------------------------------------- | -------------------------------------------- | ----------------------- |
| `id`                | The identifier for this item.                                                                   | [`String`](./string.md)                      | [`Entity`](./entity.md) |
| `suggestionStatus`  | The status of the suggestion including whether it is the original, or is accepted, or rejected. | [`SuggestionStatus`](./suggestion-status.md) | -                       |
| `authors`           | The authors of the suggestion                                                                   | [`Author`](./author.md)*                     | -                       |
| `provenance`        | A summary of the provenance of the content within the suggestion.                               | [`ProvenanceCount`](./provenance-count.md)*  | -                       |
| `executionDuration` | Time taken to generate the suggestion.                                                          | [`Duration`](./duration.md)                  | -                       |
| `executionEnded`    | The timestamp when the generation ended.                                                        | [`Timestamp`](./timestamp.md)                | -                       |
| `feedback`          | Feedback on the suggestion                                                                      | [`String`](./string.md)                      | -                       |

# Related

The `Suggestion` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: [`SuggestionBlock`](./suggestion-block.md), [`SuggestionInline`](./suggestion-inline.md)

# Bindings

The `Suggestion` type is represented in:

- [JSON-LD](https://stencila.org/Suggestion.jsonld)
- [JSON Schema](https://stencila.org/Suggestion.schema.json)
- Python class [`Suggestion`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Suggestion`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion.rs)
- TypeScript class [`Suggestion`](https://github.com/stencila/stencila/blob/main/ts/src/types/Suggestion.ts)

***

This documentation was generated from [`Suggestion.yaml`](https://github.com/stencila/stencila/blob/main/schema/Suggestion.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
