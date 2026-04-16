---
title: Citation Group
description: A group of `Citation` nodes.
---

This is a type used in Stencila Schema for grouped citations.

It exists to represent multiple related [`Citation`](./citation.md) nodes as a
single inline unit when the surrounding text cites several references at once.
This allows citation processors and renderers to generate a combined rendered
form, including ranges and grouping punctuation, while preserving the
individual citation items structurally.

Key properties include `items` and the generated `content` rendering.


# Analogues

The following external types, elements, or nodes are similar to a `CitationGroup`:

- MyST role [`cite`](https://mystmd.org/guide/roles#role-cite): Close MyST authoring analogue for grouped citations, though Stencila stores individual citation items explicitly.
- [CSL citation cluster](https://docs.citationstyles.org/en/stable/specification.html): Conceptually analogous to a rendered citation cluster in Citation Style Language processors.

# Properties

The `CitationGroup` type has these properties:

| Name      | Description                                                                 | Type                         | Inherited from          |
| --------- | --------------------------------------------------------------------------- | ---------------------------- | ----------------------- |
| `items`   | One or more `Citation`s to be referenced in the same surrounding text.      | [`Citation`](./citation.md)* | -                       |
| `content` | A rendering of the citation group using the citation style of the document. | [`Inline`](./inline.md)*     | -                       |
| `id`      | The identifier for this item.                                               | [`String`](./string.md)      | [`Entity`](./entity.md) |

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
