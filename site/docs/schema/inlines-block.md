---
title: Inlines Block
description: A block containing inlines with no other semantics.
---

This is a block type used in Stencila Schema for inline content without paragraph
semantics.

It exists to wrap inline content as a block when a container requires block
structure but a full paragraph would introduce misleading semantics or
spacing. It is especially useful during coarse decoding and similar workflows
that need a minimal block wrapper around executable inline content.

Key properties include the inline `content`.


# Analogues

The following external types, elements, or nodes are similar to a `InlinesBlock`:

- HTML [`<div>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div): Closest HTML container analogue for grouping inline content without paragraph semantics.
- Pandoc [`Plain`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Plain): Closest Pandoc block analogue for inline content that is block-positioned but not paragraph-like.
- MDAST [`Paragraph`](https://github.com/syntax-tree/mdast#paragraph): Only an approximate MDAST analogue because Markdown commonly uses paragraphs for inline block content and has no dedicated semantics-free inline block wrapper.

# Properties

The `InlinesBlock` type has these properties:

| Name      | Description                   | Type                     | Inherited from          |
| --------- | ----------------------------- | ------------------------ | ----------------------- |
| `content` | The contents of the block.    | [`Inline`](./inline.md)* | -                       |
| `id`      | The identifier for this item. | [`String`](./string.md)  | [`Entity`](./entity.md) |

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
