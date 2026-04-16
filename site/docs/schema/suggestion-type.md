---
title: Suggestion Type
description: A category of suggested edit.
---

This is an enumeration used in Stencila Schema for the kind of suggested edit.

It exists to classify whether a suggestion inserts, deletes, or modifies
content so tools can render and apply suggestions consistently.

See the type properties on [`Suggestion`](./suggestion.md) and derived
suggestion nodes for where this enumeration is used.


# Members

The `SuggestionType` type has these members:

| Member    | Description                                                           |
| --------- | --------------------------------------------------------------------- |
| `Insert`  | The suggestion is an insertion of new content.                        |
| `Delete`  | The suggestion is a deletion of existing content.                     |
| `Replace` | The suggestion is a replacement of existing content with new content. |

# Bindings

The `SuggestionType` type is represented in:

- [JSON-LD](https://stencila.org/SuggestionType.jsonld)
- [JSON Schema](https://stencila.org/SuggestionType.schema.json)
- Python type [`SuggestionType`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`SuggestionType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion_type.rs)
- TypeScript type [`SuggestionType`](https://github.com/stencila/stencila/blob/main/ts/src/types/SuggestionType.ts)

***

This documentation was generated from [`SuggestionType.yaml`](https://github.com/stencila/stencila/blob/main/schema/SuggestionType.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
