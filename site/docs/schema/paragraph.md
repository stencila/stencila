---
title: Paragraph
description: A paragraph.
---

This is a type used in Stencila Schema for paragraph content.

It exists to represent one of the main units of prose within the document
model while also supporting authorship and provenance metadata not usually
available on plain text paragraphs in simpler formats.

Key properties include the inline `content`, together with optional `authors`
and `provenance` metadata.


# Analogues

The following external types, elements, or nodes are similar to a `Paragraph`:

- HTML [`<p>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p)
- JATS [`<p>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/p.html)
- Pandoc [`Para`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Para)
- MDAST [`Paragraph`](https://github.com/syntax-tree/mdast#paragraph)

# Properties

The `Paragraph` type has these properties:

| Name         | Description                                                  | Type                                        | Inherited from          |
| ------------ | ------------------------------------------------------------ | ------------------------------------------- | ----------------------- |
| `content`    | The contents of the paragraph.                               | [`Inline`](./inline.md)*                    | -                       |
| `authors`    | The authors of the paragraph.                                | [`Author`](./author.md)*                    | -                       |
| `provenance` | A summary of the provenance of content within the paragraph. | [`ProvenanceCount`](./provenance-count.md)* | -                       |
| `id`         | The identifier for this item.                                | [`String`](./string.md)                     | [`Entity`](./entity.md) |

# Related

The `Paragraph` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Paragraph` type is represented in:

- [JSON-LD](https://stencila.org/Paragraph.jsonld)
- [JSON Schema](https://stencila.org/Paragraph.schema.json)
- Python class [`Paragraph`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Paragraph`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/paragraph.rs)
- TypeScript class [`Paragraph`](https://github.com/stencila/stencila/blob/main/ts/src/types/Paragraph.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Paragraph` type are generated using the following strategies.

::: table

| Property  | Complexity | Description                                                                     | Strategy                                      |
| --------- | ---------- | ------------------------------------------------------------------------------- | --------------------------------------------- |
| `content` | Min+       | Generate a single arbitrary inline node                                         | `vec_inlines(1)`                              |
|           | Low+       | Generate up to two arbitrary inline nodes                                       | `vec_inlines(2)`                              |
|           | High+      | Generate up to four arbitrary inline nodes                                      | `vec_inlines(4)`                              |
|           | Max        | Generate up to eight arbitrary inline nodes without restrictions on their order | `vec(Inline::arbitrary(), size_range(0..=8))` |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`Paragraph.yaml`](https://github.com/stencila/stencila/blob/main/schema/Paragraph.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
