---
title: Relative Position
description: The position of one node relative to another.
---

This is an enumeration used in Stencila Schema for describing a node's position
relative to another.

It exists to support prompt selection, editing instructions, and other
workflows that need a stable vocabulary for before, after, within, and related
positional relationships.

See properties such as
[`PromptBlock.relativePosition`](./prompt-block.md#relativeposition) for where
this enumeration is used.


# Members

The `RelativePosition` type has these members:

| Member     | Description |
| ---------- | ----------- |
| `Previous` | -           |
| `Next`     | -           |

# Bindings

The `RelativePosition` type is represented in:

- [JSON-LD](https://stencila.org/RelativePosition.jsonld)
- [JSON Schema](https://stencila.org/RelativePosition.schema.json)
- Python type [`RelativePosition`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`RelativePosition`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/relative_position.rs)
- TypeScript type [`RelativePosition`](https://github.com/stencila/stencila/blob/main/ts/src/types/RelativePosition.ts)

***

This documentation was generated from [`RelativePosition.yaml`](https://github.com/stencila/stencila/blob/main/schema/RelativePosition.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
