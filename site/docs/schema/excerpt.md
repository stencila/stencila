---
title: Excerpt
description: An excerpt from a `CreativeWork`.
---

# Properties

The `Excerpt` type has these properties:

| Name        | Description                                                          | Type                          | Inherited from          |
| ----------- | -------------------------------------------------------------------- | ----------------------------- | ----------------------- |
| `id`        | The identifier for this item.                                        | [`String`](./string.md)       | [`Entity`](./entity.md) |
| `source`    | A `Reference` to the `CreativeWork` that the excerpt was taken from. | [`Reference`](./reference.md) | -                       |
| `nodePath`  | The path to the node that was excepted.                              | [`String`](./string.md)       | -                       |
| `nodeTypes` | The types of the ancestor nodes and the node that was excerpted.     | [`String`](./string.md)       | -                       |
| `content`   | The excerpted content.                                               | [`Block`](./block.md)*        | -                       |

# Related

The `Excerpt` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Excerpt` type is represented in:

- [JSON-LD](https://stencila.org/Excerpt.jsonld)
- [JSON Schema](https://stencila.org/Excerpt.schema.json)
- Python class [`Excerpt`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/excerpt.py)
- Rust struct [`Excerpt`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/excerpt.rs)
- TypeScript class [`Excerpt`](https://github.com/stencila/stencila/blob/main/ts/src/types/Excerpt.ts)

# Source

This documentation was generated from [`Excerpt.yaml`](https://github.com/stencila/stencila/blob/main/schema/Excerpt.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
