---
title: Island
description: An island of content in a document.
---

This is a type used in Stencila Schema for an isolated region of document content.

It exists to represent content that is handled with its own compilation,
rendering, or interaction boundary while still remaining inside the broader
document model. This is useful for embedded applications, custom renderers, or
sandboxed content regions.

Key properties describe the island content, source, and any execution or
rendering context associated with it.


# Properties

The `Island` type has these properties:

| Name                 | Description                                                | Type                           | Inherited from          |
| -------------------- | ---------------------------------------------------------- | ------------------------------ | ----------------------- |
| `content`            | The content within the section.                            | [`Block`](./block.md)*         | -                       |
| `isAutomatic`        | Whether the island is automatically generated.             | [`Boolean`](./boolean.md)      | -                       |
| `isContinuation`     | Whether the island is a continuation of a previous island. | [`Boolean`](./boolean.md)      | -                       |
| `labelType`          | The type of the label for the island.                      | [`LabelType`](./label-type.md) | -                       |
| `label`              | A short label for the chunk.                               | [`String`](./string.md)        | -                       |
| `labelAutomatically` | Whether the label should be automatically updated.         | [`Boolean`](./boolean.md)      | -                       |
| `otherIds`           | Other IDs for the island, in addition to the primary `id`. | [`String`](./string.md)*       | -                       |
| `style`              | The style to apply to the island.                          | [`String`](./string.md)        | -                       |
| `id`                 | The identifier for this item.                              | [`String`](./string.md)        | [`Entity`](./entity.md) |

# Related

The `Island` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Island` type is represented in:

- [JSON-LD](https://stencila.org/Island.jsonld)
- [JSON Schema](https://stencila.org/Island.schema.json)
- Python class [`Island`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Island`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/island.rs)
- TypeScript class [`Island`](https://github.com/stencila/stencila/blob/main/ts/src/types/Island.ts)

***

This documentation was generated from [`Island.yaml`](https://github.com/stencila/stencila/blob/main/schema/Island.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
