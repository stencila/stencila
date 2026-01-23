---
title: Provenance Count
description: The count of the number of characters in a `ProvenanceCategory` within an entity.
---

# Properties

The `ProvenanceCount` type has these properties:

| Name                 | Description                                                  | Type                                             | Inherited from          |
| -------------------- | ------------------------------------------------------------ | ------------------------------------------------ | ----------------------- |
| `id`                 | The identifier for this item.                                | [`String`](./string.md)                          | [`Entity`](./entity.md) |
| `provenanceCategory` | The provenance category that the character count applies to. | [`ProvenanceCategory`](./provenance-category.md) | -                       |
| `characterCount`     | The number of characters in the provenance category.         | [`UnsignedInteger`](./unsigned-integer.md)       | -                       |
| `characterPercent`   | The percentage of characters in the provenance category.     | [`UnsignedInteger`](./unsigned-integer.md)       | -                       |

# Related

The `ProvenanceCount` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ProvenanceCount` type is represented in:

- [JSON-LD](https://stencila.org/ProvenanceCount.jsonld)
- [JSON Schema](https://stencila.org/ProvenanceCount.schema.json)
- Python class [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/provenance_count.rs)
- TypeScript class [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/ts/src/types/ProvenanceCount.ts)

***

This documentation was generated from [`ProvenanceCount.yaml`](https://github.com/stencila/stencila/blob/main/schema/ProvenanceCount.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
