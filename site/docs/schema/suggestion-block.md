---
title: Suggestion Block
description: Abstract base type for nodes that indicate a suggested change to block content.
---

# Properties

The `SuggestionBlock` type has these properties:

| Name                | Description                                                                                     | Type                                         | Inherited from                  |
| ------------------- | ----------------------------------------------------------------------------------------------- | -------------------------------------------- | ------------------------------- |
| `id`                | The identifier for this item.                                                                   | [`String`](./string.md)                      | [`Entity`](./entity.md)         |
| `suggestionStatus`  | The status of the suggestion including whether it is the original, or is accepted, or rejected. | [`SuggestionStatus`](./suggestion-status.md) | [`Suggestion`](./suggestion.md) |
| `authors`           | The authors of the suggestion                                                                   | [`Author`](./author.md)*                     | [`Suggestion`](./suggestion.md) |
| `provenance`        | A summary of the provenance of the content within the suggestion.                               | [`ProvenanceCount`](./provenance-count.md)*  | [`Suggestion`](./suggestion.md) |
| `executionDuration` | Time taken to generate the suggestion.                                                          | [`Duration`](./duration.md)                  | [`Suggestion`](./suggestion.md) |
| `executionEnded`    | The timestamp when the generation ended.                                                        | [`Timestamp`](./timestamp.md)                | [`Suggestion`](./suggestion.md) |
| `feedback`          | Feedback on the suggestion                                                                      | [`String`](./string.md)                      | [`Suggestion`](./suggestion.md) |
| `content`           | The content that is suggested to be inserted, modified, replaced, or deleted.                   | [`Block`](./block.md)*                       | -                               |

# Related

The `SuggestionBlock` type is related to these types:

- Parents: [`Suggestion`](./suggestion.md)
- Children: none

# Bindings

The `SuggestionBlock` type is represented in:

- [JSON-LD](https://stencila.org/SuggestionBlock.jsonld)
- [JSON Schema](https://stencila.org/SuggestionBlock.schema.json)
- Python class [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/suggestion_block.py)
- Rust struct [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion_block.rs)
- TypeScript class [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/SuggestionBlock.ts)

# Source

This documentation was generated from [`SuggestionBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/SuggestionBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
