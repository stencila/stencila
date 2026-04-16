---
title: Suggestion Block
description: Abstract base type for nodes that indicate a suggested change to block content.
---

This is an abstract base type used in Stencila Schema for suggestions that operate on
block content.

It extends [`Suggestion`](./suggestion.md) so proposed changes to block-level
nodes can share common suggestion behavior while remaining distinct from
inline suggestions.

Key properties are primarily inherited from [`Suggestion`](./suggestion.md),
with semantics centered on block-content replacements and originals.


# Analogues

The following external types, elements, or nodes are similar to a `SuggestionBlock`:

- [block-level tracked change](https://learn.microsoft.com/en-us/office/open-xml/word/working-with-comments-and-tracked-revisions): Approximate analogue for suggested paragraph or block replacements in tracked-review systems.

# Properties

The `SuggestionBlock` type has these properties:

| Name                | Description                                                                                                                        | Type                                         | Inherited from                  |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- | ------------------------------- |
| `content`           | The suggested content. For insertions and replacements, this is the new content; for deletions, this is the content being deleted. | [`Block`](./block.md)*                       | -                               |
| `original`          | The original content. For replacements, this is the content being replaced; for deletions, this should be absent.                  | [`Block`](./block.md)*                       | -                               |
| `suggestionType`    | The type of suggestion including whether it is an insertion or a deletion.                                                         | [`SuggestionType`](./suggestion-type.md)     | [`Suggestion`](./suggestion.md) |
| `suggestionStatus`  | The status of the suggestion including whether it is the original, or is accepted, or rejected.                                    | [`SuggestionStatus`](./suggestion-status.md) | [`Suggestion`](./suggestion.md) |
| `authors`           | The authors of the suggestion                                                                                                      | [`Author`](./author.md)*                     | [`Suggestion`](./suggestion.md) |
| `provenance`        | A summary of the provenance of the content within the suggestion.                                                                  | [`ProvenanceCount`](./provenance-count.md)*  | [`Suggestion`](./suggestion.md) |
| `executionDuration` | Time taken to generate the suggestion.                                                                                             | [`Duration`](./duration.md)                  | [`Suggestion`](./suggestion.md) |
| `executionEnded`    | The timestamp when the generation ended.                                                                                           | [`Timestamp`](./timestamp.md)                | [`Suggestion`](./suggestion.md) |
| `feedback`          | Feedback on the suggestion                                                                                                         | [`String`](./string.md)                      | [`Suggestion`](./suggestion.md) |
| `id`                | The identifier for this item.                                                                                                      | [`String`](./string.md)                      | [`Entity`](./entity.md)         |

# Related

The `SuggestionBlock` type is related to these types:

- Parents: [`Suggestion`](./suggestion.md)
- Children: none

# Bindings

The `SuggestionBlock` type is represented in:

- [JSON-LD](https://stencila.org/SuggestionBlock.jsonld)
- [JSON Schema](https://stencila.org/SuggestionBlock.schema.json)
- Python class [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion_block.rs)
- TypeScript class [`SuggestionBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/SuggestionBlock.ts)

***

This documentation was generated from [`SuggestionBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/SuggestionBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
