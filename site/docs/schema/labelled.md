---
title: Labelled
description: Abstract base for document nodes with labels.
---

This is a mixin-style base type for document nodes that participate in
author-facing numbering, captions, and cross-references. It centralizes the
common label metadata and whether an identifier was generated automatically.


# Properties

The `Labelled` type has these properties:

| Name                 | Description                                             | Type                      | Inherited from |
| -------------------- | ------------------------------------------------------- | ------------------------- | -------------- |
| `idAutomatically`    | Whether the identifier should be automatically updated. | [`Boolean`](./boolean.md) | -              |
| `label`              | A short label for the node.                             | [`String`](./string.md)   | -              |
| `labelAutomatically` | Whether the label should be automatically updated.      | [`Boolean`](./boolean.md) | -              |

# Related

The `Labelled` type is related to these types:

- Parents: None
- Children: [`CodeChunk`](./code-chunk.md), [`Datatable`](./datatable.md), [`Figure`](./figure.md), [`Island`](./island.md), [`MathBlock`](./math-block.md), [`Supplement`](./supplement.md), [`Table`](./table.md)

# Bindings

The `Labelled` type is represented in:

- [JSON-LD](https://stencila.org/Labelled.jsonld)
- [JSON Schema](https://stencila.org/Labelled.schema.json)
- Python class [`Labelled`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Labelled`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/labelled.rs)
- TypeScript class [`Labelled`](https://github.com/stencila/stencila/blob/main/ts/src/types/Labelled.ts)

***

This documentation was generated from [`Labelled.yaml`](https://github.com/stencila/stencila/blob/main/schema/Labelled.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
