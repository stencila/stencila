---
title: Thematic Break
description: A thematic break.
---

This is a block type used in Stencila Schema for thematic or scene breaks, analogous
to HTML `<hr>` and Markdown thematic breaks.

It exists to preserve this structural break as an explicit node within the
document model rather than as a formatting artifact.

This type is mainly useful as a block separator within prose-oriented content
models such as [`Block`](./block.md).


# Analogues

The following external types, elements, or nodes are similar to a `ThematicBreak`:

- HTML [`<hr>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hr)
- JATS [`<hr>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/hr.html)
- Pandoc [`HorizontalRule`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:HorizontalRule)
- MDAST [`ThematicBreak`](https://github.com/syntax-tree/mdast#thematicbreak)

# Properties

The `ThematicBreak` type has these properties:

| Name | Description                   | Type                    | Inherited from          |
| ---- | ----------------------------- | ----------------------- | ----------------------- |
| `id` | The identifier for this item. | [`String`](./string.md) | [`Entity`](./entity.md) |

# Related

The `ThematicBreak` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ThematicBreak` type is represented in:

- [JSON-LD](https://stencila.org/ThematicBreak.jsonld)
- [JSON Schema](https://stencila.org/ThematicBreak.schema.json)
- Python class [`ThematicBreak`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ThematicBreak`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/thematic_break.rs)
- TypeScript class [`ThematicBreak`](https://github.com/stencila/stencila/blob/main/ts/src/types/ThematicBreak.ts)

***

This documentation was generated from [`ThematicBreak.yaml`](https://github.com/stencila/stencila/blob/main/schema/ThematicBreak.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
