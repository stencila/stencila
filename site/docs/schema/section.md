---
title: Section
description: A section of a document.
---

# Properties

The `Section` type has these properties:

| Name          | Description                                                    | Type                                        | Inherited from          |
| ------------- | -------------------------------------------------------------- | ------------------------------------------- | ----------------------- |
| `id`          | The identifier for this item.                                  | [`String`](./string.md)                     | [`Entity`](./entity.md) |
| `sectionType` | The type of section.                                           | [`SectionType`](./section-type.md)          | -                       |
| `content`     | The content within the section.                                | [`Block`](./block.md)*                      | -                       |
| `authors`     | The authors of the section.                                    | [`Author`](./author.md)*                    | -                       |
| `provenance`  | A summary of the provenance of the content within the section. | [`ProvenanceCount`](./provenance-count.md)* | -                       |

# Related

The `Section` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Section` type is represented in:

- [JSON-LD](https://stencila.org/Section.jsonld)
- [JSON Schema](https://stencila.org/Section.schema.json)
- Python class [`Section`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/section.py)
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

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`Section.yaml`](https://github.com/stencila/stencila/blob/main/schema/Section.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
