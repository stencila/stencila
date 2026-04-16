---
title: Suggestion Inline
description: Abstract base type for nodes that indicate a suggested change to inline content.
---

This is an abstract base type used in Stencila Schema for suggestions that operate on
inline content.

It extends [`Suggestion`](./suggestion.md) so fine-grained proposed edits
within prose can be represented separately from block-level changes while
reusing the common suggestion model.

Key properties are primarily inherited from [`Suggestion`](./suggestion.md),
with semantics centered on inline-content replacements and originals.


# Analogues

The following external types, elements, or nodes are similar to a `SuggestionInline`:

- [inline tracked change](https://learn.microsoft.com/en-us/office/open-xml/word/working-with-comments-and-tracked-revisions): Approximate analogue for suggested insertions, deletions, and replacements within prose spans.

# Properties

The `SuggestionInline` type has these properties:

| Name                | Description                                                                                                                        | Type                                         | Inherited from                  |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- | ------------------------------- |
| `content`           | The suggested content. For insertions and replacements, this is the new content; for deletions, this is the content being deleted. | [`Inline`](./inline.md)*                     | -                               |
| `original`          | The original content. For replacements, this is the content being replaced; for deletions, this should be absent.                  | [`Inline`](./inline.md)*                     | -                               |
| `suggestionType`    | The type of suggestion including whether it is an insertion or a deletion.                                                         | [`SuggestionType`](./suggestion-type.md)     | [`Suggestion`](./suggestion.md) |
| `suggestionStatus`  | The status of the suggestion including whether it is the original, or is accepted, or rejected.                                    | [`SuggestionStatus`](./suggestion-status.md) | [`Suggestion`](./suggestion.md) |
| `authors`           | The authors of the suggestion                                                                                                      | [`Author`](./author.md)*                     | [`Suggestion`](./suggestion.md) |
| `provenance`        | A summary of the provenance of the content within the suggestion.                                                                  | [`ProvenanceCount`](./provenance-count.md)*  | [`Suggestion`](./suggestion.md) |
| `executionDuration` | Time taken to generate the suggestion.                                                                                             | [`Duration`](./duration.md)                  | [`Suggestion`](./suggestion.md) |
| `executionEnded`    | The timestamp when the generation ended.                                                                                           | [`Timestamp`](./timestamp.md)                | [`Suggestion`](./suggestion.md) |
| `feedback`          | Feedback on the suggestion                                                                                                         | [`String`](./string.md)                      | [`Suggestion`](./suggestion.md) |
| `id`                | The identifier for this item.                                                                                                      | [`String`](./string.md)                      | [`Entity`](./entity.md)         |

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
