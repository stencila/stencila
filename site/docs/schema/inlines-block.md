---
title: Inlines Block
description: A block containing inlines with no other semantics.
---

Similar in structure to a `Paragraph` but displayed without newline or other spacing around it.
Used when decoding content with the `--coarse` option to encapsulate executable inlines without
creating a new paragraph.


# Properties

The `InlinesBlock` type has these properties:

| Name      | Description                   | Type                     | Inherited from          |
| --------- | ----------------------------- | ------------------------ | ----------------------- |
| `id`      | The identifier for this item. | [`String`](./string.md)  | [`Entity`](./entity.md) |
| `content` | The contents of the block.    | [`Inline`](./inline.md)* | -                       |

# Related

The `InlinesBlock` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `InlinesBlock` type is represented in:

- [JSON-LD](https://stencila.org/InlinesBlock.jsonld)
- [JSON Schema](https://stencila.org/InlinesBlock.schema.json)
- Python class [`InlinesBlock`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`InlinesBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/inlines_block.rs)
- TypeScript class [`InlinesBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/InlinesBlock.ts)

***

This documentation was generated from [`InlinesBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/InlinesBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
