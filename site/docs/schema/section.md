---
title: Section
description: A section of a document.
---

This is a type used in Stencila Schema for document sections, analogous to section
containers in HTML, JATS, and other structured document models.

It exists to group block content under headings while carrying structured
metadata, authorship, and provenance. Unlike a plain heading hierarchy
inferred from text alone, explicit sections allow more reliable transformation
and editing workflows.

Key properties include `sectionType`, `title`, and `content`, together with
inherited metadata from [`Entity`](./entity.md).


# Analogues

The following external types, elements, or nodes are similar to a `Section`:

- HTML [`<section>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/section): Closest HTML sectioning analogue, though Stencila sections can carry typed section roles, authorship, and provenance metadata.
- JATS [`<sec>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/sec.html): Closest JATS analogue for structured document sections.
- Pandoc [`Div`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Div): Closest Pandoc container analogue when divisions are used to represent sections; Stencila uses a dedicated section node with explicit semantics.
- MDAST [`Heading`](https://github.com/syntax-tree/mdast#heading): Markdown ASTs often imply sections through heading structure rather than an explicit section container, so this is only an approximate analogue.

# Properties

The `Section` type has these properties:

| Name          | Description                                                    | Type                                        | Inherited from          |
| ------------- | -------------------------------------------------------------- | ------------------------------------------- | ----------------------- |
| `sectionType` | The type of section.                                           | [`SectionType`](./section-type.md)          | -                       |
| `content`     | The content within the section.                                | [`Block`](./block.md)*                      | -                       |
| `authors`     | The authors of the section.                                    | [`Author`](./author.md)*                    | -                       |
| `provenance`  | A summary of the provenance of the content within the section. | [`ProvenanceCount`](./provenance-count.md)* | -                       |
| `id`          | The identifier for this item.                                  | [`String`](./string.md)                     | [`Entity`](./entity.md) |

# Related

The `Section` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Section` type is represented in:

- [JSON-LD](https://stencila.org/Section.jsonld)
- [JSON Schema](https://stencila.org/Section.schema.json)
- Python class [`Section`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Section`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/section.rs)
- TypeScript class [`Section`](https://github.com/stencila/stencila/blob/main/ts/src/types/Section.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Section` type are generated using the following strategies.

::: table

| Property      | Complexity | Description                                                 | Strategy                               |
| ------------- | ---------- | ----------------------------------------------------------- | -------------------------------------- |
| `sectionType` | Min+       | No type.                                                    | `None`                                 |
|               | Low+       | Generate an arbitrary section type.                         | `option::of(SectionType::arbitrary())` |
| `content`     | Min+       | An empty vector                                             | `Vec::new()`                           |
|               | Low+       | Generate an arbitrary heading and an arbitrary paragraph.   | `vec_heading_paragraph()`              |
|               | High+      | Generate up to four arbitrary, non-recursive, block nodes.  | `vec_blocks_non_recursive(4)`          |
|               | Max        | Generate up to eight arbitrary, non-recursive, block nodes. | `vec_blocks_non_recursive(8)`          |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`Section.yaml`](https://github.com/stencila/stencila/blob/main/schema/Section.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
