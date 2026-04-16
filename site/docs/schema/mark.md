---
title: Mark
description: An abstract base type for marked inline content.
---

This is an abstract base type used in Stencila Schema for inline marked content.

It exists to group inline nodes that wrap other inline content with semantic
markup such as emphasis, quotes, annotations, or editorial marks. This allows
shared handling of nested inline content and common metadata while keeping
specific mark semantics in derived types.

Key properties usually include wrapped `content`, together with inherited
metadata and provenance features from [`Entity`](./entity.md).


# Analogues

The following external types, elements, or nodes are similar to a `Mark`:

- [HTML phrasing elements](https://html.spec.whatwg.org/multipage/dom.html#phrasing-content-2): Broadly analogous to inline wrapper elements such as `<em>`, `<strong>`, or `<q>`, though Stencila abstracts over many specific mark types.
- Pandoc [`Inline`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#t:Inline): Approximate Pandoc analogue because concrete mark types are individual inline constructors rather than subclasses of an abstract mark base type.

# Properties

The `Mark` type has these properties:

| Name      | Description                   | Type                     | Inherited from          |
| --------- | ----------------------------- | ------------------------ | ----------------------- |
| `content` | The content that is marked.   | [`Inline`](./inline.md)* | -                       |
| `id`      | The identifier for this item. | [`String`](./string.md)  | [`Entity`](./entity.md) |

# Related

The `Mark` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: [`Annotation`](./annotation.md), [`Emphasis`](./emphasis.md), [`QuoteInline`](./quote-inline.md), [`Strikeout`](./strikeout.md), [`Strong`](./strong.md), [`Subscript`](./subscript.md), [`Superscript`](./superscript.md), [`Underline`](./underline.md)

# Bindings

The `Mark` type is represented in:

- [JSON-LD](https://stencila.org/Mark.jsonld)
- [JSON Schema](https://stencila.org/Mark.schema.json)
- Python class [`Mark`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Mark`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/mark.rs)
- TypeScript class [`Mark`](https://github.com/stencila/stencila/blob/main/ts/src/types/Mark.ts)

***

This documentation was generated from [`Mark.yaml`](https://github.com/stencila/stencila/blob/main/schema/Mark.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
