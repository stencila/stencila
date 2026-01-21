---
title: Annotation
description: Annotated content.
---

# Properties

The `Annotation` type has these properties:

| Name         | Description                          | Type                     | Inherited from          |
| ------------ | ------------------------------------ | ------------------------ | ----------------------- |
| `id`         | The identifier for this item.        | [`String`](./string.md)  | [`Entity`](./entity.md) |
| `content`    | The content that is marked.          | [`Inline`](./inline.md)* | [`Mark`](./mark.md)     |
| `annotation` | The annotation, usually a paragraph. | [`Block`](./block.md)*   | -                       |

# Related

The `Annotation` type is related to these types:

- Parents: [`Mark`](./mark.md)
- Children: none

# Bindings

The `Annotation` type is represented in:

- [JSON-LD](https://stencila.org/Annotation.jsonld)
- [JSON Schema](https://stencila.org/Annotation.schema.json)
- Python class [`Annotation`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/annotation.py)
- Rust struct [`Annotation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/annotation.rs)
- TypeScript class [`Annotation`](https://github.com/stencila/stencila/blob/main/ts/src/types/Annotation.ts)

# Source

This documentation was generated from [`Annotation.yaml`](https://github.com/stencila/stencila/blob/main/schema/Annotation.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
