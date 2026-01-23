---
title: Supplement
description: A supplementary `CreativeWork` that supports this work but is not considered part of its main content.
---

Corresponds to the JATS `<supplementary-material>` element 
(https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/supplementary-material.html).

As in JATS, this is a `Block` content type so that supplementary material 
can be positioned close to the content it relates to (e.g., within a figure caption) 
rather than only at the end of an article. Nevertheless, many articles 
will include a dedicated "Supplementary Materials" section composed of a `Heading` 
followed by one or more `Supplement` blocks.


# Properties

The `Supplement` type has these properties:

| Name                  | Description                                                                 | Type                                                | Inherited from          |
| --------------------- | --------------------------------------------------------------------------- | --------------------------------------------------- | ----------------------- |
| `id`                  | The identifier for this item.                                               | [`String`](./string.md)                             | [`Entity`](./entity.md) |
| `workType`            | The `CreativeWork` type of the supplement.                                  | [`CreativeWorkType`](./creative-work-type.md)       | -                       |
| `label`               | A short identifier or title for the supplement (e.g., "S1").                | [`String`](./string.md)                             | -                       |
| `labelAutomatically`  | Whether the supplement label should be automatically generated and updated. | [`Boolean`](./boolean.md)                           | -                       |
| `caption`             | A brief caption or description for the supplement.                          | [`Block`](./block.md)*                              | -                       |
| `target`              | A reference to the supplement.                                              | [`String`](./string.md)                             | -                       |
| `compilationMessages` | Any messages generated while embedding the supplement.                      | [`CompilationMessage`](./compilation-message.md)*   | -                       |
| `work`                | The `CreativeWork` that constitutes the supplement.                         | [`CreativeWorkVariant`](./creative-work-variant.md) | -                       |

# Related

The `Supplement` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Supplement` type is represented in:

- [JSON-LD](https://stencila.org/Supplement.jsonld)
- [JSON Schema](https://stencila.org/Supplement.schema.json)
- Python class [`Supplement`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Supplement`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/supplement.rs)
- TypeScript class [`Supplement`](https://github.com/stencila/stencila/blob/main/ts/src/types/Supplement.ts)

***

This documentation was generated from [`Supplement.yaml`](https://github.com/stencila/stencila/blob/main/schema/Supplement.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
