---
title: Suggestion Status
description: A review status for a suggestion.
---

This is an enumeration used in Stencila Schema for the status of a suggestion.

It exists so review workflows can distinguish proposed, accepted, rejected,
and related states using a stable vocabulary across tools and formats.

See the status properties on [`Suggestion`](./suggestion.md) and related
editing types for where this enumeration is used.


# Members

The `SuggestionStatus` type has these members:

| Member     | Description                             |
| ---------- | --------------------------------------- |
| `Original` | The suggestion is the original content. |
| `Accepted` | The suggestion has been accepted.       |
| `Rejected` | The suggestion has been rejected.       |

# Bindings

The `SuggestionStatus` type is represented in:

- [JSON-LD](https://stencila.org/SuggestionStatus.jsonld)
- [JSON Schema](https://stencila.org/SuggestionStatus.schema.json)
- Python type [`SuggestionStatus`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`SuggestionStatus`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion_status.rs)
- TypeScript type [`SuggestionStatus`](https://github.com/stencila/stencila/blob/main/ts/src/types/SuggestionStatus.ts)

***

This documentation was generated from [`SuggestionStatus.yaml`](https://github.com/stencila/stencila/blob/main/schema/SuggestionStatus.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
