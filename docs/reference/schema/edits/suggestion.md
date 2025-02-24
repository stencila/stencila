---
title: Suggestion
description: Abstract base type for nodes that indicate a suggested change to content.
config:
  publish:
    ghost:
      type: post
      slug: suggestion
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Edits
---

# Properties

The `Suggestion` type has these properties:

| Name                | Description                                                                                     | Type                                                                                    | Inherited from                                                     | `JSON-LD @id`                                | Aliases                                    |
| ------------------- | ----------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------- | ------------------------------------------ |
| `id`                | The identifier for this item.                                                                   | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                      | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)         | -                                          |
| `suggestionStatus`  | The status of the suggestion including whether it is the original, or is accepted, or rejected. | [`SuggestionStatus`](https://stencila.ghost.io/docs/reference/schema/suggestion-status) | -                                                                  | `stencila:suggestionStatus`                  | `suggestion-status`, `suggestion_status`   |
| `authors`           | The authors of the suggestion                                                                   | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                     | -                                                                  | [`schema:author`](https://schema.org/author) | `author`                                   |
| `provenance`        | A summary of the provenance of the content within the suggestion.                               | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)*  | -                                                                  | `stencila:provenance`                        | -                                          |
| `executionDuration` | Time taken to generate the suggestion.                                                          | [`Duration`](https://stencila.ghost.io/docs/reference/schema/duration)                  | -                                                                  | `stencila:executionDuration`                 | `execution-duration`, `execution_duration` |
| `executionEnded`    | The timestamp when the generation ended.                                                        | [`Timestamp`](https://stencila.ghost.io/docs/reference/schema/timestamp)                | -                                                                  | `stencila:executionEnded`                    | `execution-ended`, `execution_ended`       |
| `feedback`          | Feedback on the suggestion                                                                      | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                      | -                                                                  | `stencila:feedback`                          | -                                          |

# Related

The `Suggestion` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: [`SuggestionBlock`](https://stencila.ghost.io/docs/reference/schema/suggestion-block), [`SuggestionInline`](https://stencila.ghost.io/docs/reference/schema/suggestion-inline)

# Bindings

The `Suggestion` type is represented in:

- [JSON-LD](https://stencila.org/Suggestion.jsonld)
- [JSON Schema](https://stencila.org/Suggestion.schema.json)
- Python class [`Suggestion`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/suggestion.py)
- Rust struct [`Suggestion`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion.rs)
- TypeScript class [`Suggestion`](https://github.com/stencila/stencila/blob/main/ts/src/types/Suggestion.ts)

# Source

This documentation was generated from [`Suggestion.yaml`](https://github.com/stencila/stencila/blob/main/schema/Suggestion.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
