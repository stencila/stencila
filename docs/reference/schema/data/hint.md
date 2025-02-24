---
title: Hint
description: Union type for hints of the value and/or structure of data.
config:
  publish:
    ghost:
      type: post
      slug: hint
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Data
---

# Members

The `Hint` type has these members:

- [`ArrayHint`](https://stencila.ghost.io/docs/reference/schema/array-hint)
- [`DatatableHint`](https://stencila.ghost.io/docs/reference/schema/datatable-hint)
- [`Function`](https://stencila.ghost.io/docs/reference/schema/function)
- [`ObjectHint`](https://stencila.ghost.io/docs/reference/schema/object-hint)
- [`StringHint`](https://stencila.ghost.io/docs/reference/schema/string-hint)
- [`Unknown`](https://stencila.ghost.io/docs/reference/schema/unknown)
- [`Boolean`](https://stencila.ghost.io/docs/reference/schema/boolean)
- [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)
- [`Number`](https://stencila.ghost.io/docs/reference/schema/number)

# Bindings

The `Hint` type is represented in:

- [JSON-LD](https://stencila.org/Hint.jsonld)
- [JSON Schema](https://stencila.org/Hint.schema.json)
- Python type [`Hint`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/hint.py)
- Rust type [`Hint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/hint.rs)
- TypeScript type [`Hint`](https://github.com/stencila/stencila/blob/main/ts/src/types/Hint.ts)

# Source

This documentation was generated from [`Hint.yaml`](https://github.com/stencila/stencila/blob/main/schema/Hint.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
