---
title: Excerpt
description: An excerpt from a `CreativeWork`.
---

This is a type used in Stencila Schema for representing an excerpt from another
creative work.

It exists to preserve the distinction between quoted or extracted material and
original document content, while allowing references back to the source work.
This is useful in research, review, and annotation workflows where excerpts
need structured attribution.

Key properties include the excerpted `content` and links to the source work or
location.


# Properties

The `Excerpt` type has these properties:

| Name        | Description                                                          | Type                          | Inherited from          |
| ----------- | -------------------------------------------------------------------- | ----------------------------- | ----------------------- |
| `source`    | A `Reference` to the `CreativeWork` that the excerpt was taken from. | [`Reference`](./reference.md) | -                       |
| `nodePath`  | The path to the node that was excepted.                              | [`String`](./string.md)       | -                       |
| `nodeTypes` | The types of the ancestor nodes and the node that was excerpted.     | [`String`](./string.md)       | -                       |
| `content`   | The excerpted content.                                               | [`Block`](./block.md)*        | -                       |
| `id`        | The identifier for this item.                                        | [`String`](./string.md)       | [`Entity`](./entity.md) |

# Related

The `Excerpt` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Excerpt` type is represented in:

- [JSON-LD](https://stencila.org/Excerpt.jsonld)
- [JSON Schema](https://stencila.org/Excerpt.schema.json)
- Python class [`Excerpt`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Excerpt`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/excerpt.rs)
- TypeScript class [`Excerpt`](https://github.com/stencila/stencila/blob/main/ts/src/types/Excerpt.ts)

***

This documentation was generated from [`Excerpt.yaml`](https://github.com/stencila/stencila/blob/main/schema/Excerpt.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
