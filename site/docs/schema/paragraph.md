---
title: Paragraph
description: A paragraph.
---

Analogues of `Paragraph` in other schema include:
  - HTML [`<p>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p)
  - JATS XML [`<p>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/p.html)
  - MDAST [`Paragraph`](https://github.com/syntax-tree/mdast#Paragraph)
  - OpenDocument [`<text:p>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415138_253892949)
  - Pandoc [`Para`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L220)


# Properties

The `Paragraph` type has these properties:

| Name         | Description                                                  | Type                                        | Inherited from          |
| ------------ | ------------------------------------------------------------ | ------------------------------------------- | ----------------------- |
| `id`         | The identifier for this item.                                | [`String`](./string.md)                     | [`Entity`](./entity.md) |
| `content`    | The contents of the paragraph.                               | [`Inline`](./inline.md)*                    | -                       |
| `authors`    | The authors of the paragraph.                                | [`Author`](./author.md)*                    | -                       |
| `provenance` | A summary of the provenance of content within the paragraph. | [`ProvenanceCount`](./provenance-count.md)* | -                       |

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
