---
title: Supplement
description: A supplementary creative work associated with a document.
---

This is a Stencila-native type corresponding closely to the JATS
[`<supplementary-material>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/supplementary-material.html)
element.

It is used to attach supplementary works to a document while still allowing
them to appear as block content near the material they support, such as within
a figure caption or a dedicated supplementary materials section. This lets
Stencila model supplements as document content rather than only as distant
metadata attachments.

Key properties include `workType`, `label`, `caption`, and `target`.


# Analogues

The following external types, elements, or nodes are similar to a `Supplement`:

- JATS [`<supplementary-material>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/supplementary-material.html)

# Properties

The `Supplement` type has these properties:

| Name                  | Description                                                                 | Type                                                | Inherited from          |
| --------------------- | --------------------------------------------------------------------------- | --------------------------------------------------- | ----------------------- |
| `workType`            | The `CreativeWork` type of the supplement.                                  | [`CreativeWorkType`](./creative-work-type.md)       | -                       |
| `label`               | A short identifier or title for the supplement (e.g., "S1").                | [`String`](./string.md)                             | -                       |
| `labelAutomatically`  | Whether the supplement label should be automatically generated and updated. | [`Boolean`](./boolean.md)                           | -                       |
| `caption`             | A brief caption or description for the supplement.                          | [`Block`](./block.md)*                              | -                       |
| `target`              | A reference to the supplement.                                              | [`String`](./string.md)                             | -                       |
| `compilationMessages` | Any messages generated while embedding the supplement.                      | [`CompilationMessage`](./compilation-message.md)*   | -                       |
| `work`                | The `CreativeWork` that constitutes the supplement.                         | [`CreativeWorkVariant`](./creative-work-variant.md) | -                       |
| `id`                  | The identifier for this item.                                               | [`String`](./string.md)                             | [`Entity`](./entity.md) |

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
