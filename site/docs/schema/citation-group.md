---
title: Citation Group
description: A group of `Citation` nodes.
---

This type allows you to group associated citations together.
When some content in a [`CreativeWork`](./creative-work.md) cites more than one
reference for a particular piece of text, use a `CitationGroup` to encapsulate
multiple [`Citation`](./citation.md) nodes.

At present we do not give a `citationMode` property to a `CitationGroup` since
they will almost always be parenthetical as opposed to narrative.
In other words, it usually only makes sense for individual `Citation` nodes to be
narrative (although they may be connected together within `content` using words
such as "and").


# Properties

The `CitationGroup` type has these properties:

| Name      | Description                                                                 | Type                         | Inherited from          |
| --------- | --------------------------------------------------------------------------- | ---------------------------- | ----------------------- |
| `id`      | The identifier for this item.                                               | [`String`](./string.md)      | [`Entity`](./entity.md) |
| `items`   | One or more `Citation`s to be referenced in the same surrounding text.      | [`Citation`](./citation.md)* | -                       |
| `content` | A rendering of the citation group using the citation style of the document. | [`Inline`](./inline.md)*     | -                       |

# Related

The `CitationGroup` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `CitationGroup` type is represented in:

- [JSON-LD](https://stencila.org/CitationGroup.jsonld)
- [JSON Schema](https://stencila.org/CitationGroup.schema.json)
- Python class [`CitationGroup`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`CitationGroup`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/citation_group.rs)
- TypeScript class [`CitationGroup`](https://github.com/stencila/stencila/blob/main/ts/src/types/CitationGroup.ts)

***

This documentation was generated from [`CitationGroup.yaml`](https://github.com/stencila/stencila/blob/main/schema/CitationGroup.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
