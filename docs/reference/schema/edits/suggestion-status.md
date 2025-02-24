---
title: Suggestion Status
description: The status of an instruction.
config:
  publish:
    ghost:
      type: post
      slug: suggestion-status
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Edits
---

# Members

The `SuggestionStatus` type has these members:

- `Original`
- `Accepted`
- `Rejected`

# Bindings

The `SuggestionStatus` type is represented in:

- [JSON-LD](https://stencila.org/SuggestionStatus.jsonld)
- [JSON Schema](https://stencila.org/SuggestionStatus.schema.json)
- Python type [`SuggestionStatus`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/suggestion_status.py)
- Rust type [`SuggestionStatus`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion_status.rs)
- TypeScript type [`SuggestionStatus`](https://github.com/stencila/stencila/blob/main/ts/src/types/SuggestionStatus.ts)

# Source

This documentation was generated from [`SuggestionStatus.yaml`](https://github.com/stencila/stencila/blob/main/schema/SuggestionStatus.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
