---
title: Heading
description: A heading.
---

This is a type used in Stencila Schema for document headings.

It exists to represent heading content and hierarchy explicitly, with
additional support for document labeling, authorship, and provenance. This
makes headings more than presentational text and allows them to participate in
sectioning, appendix numbering, and document navigation workflows.

Key properties include `level`, `content`, `labelType`, and `label`.


# Analogues

The following external types, elements, or nodes are similar to a `Heading`:

- HTML [`<h1>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h1): Closest HTML analogue for heading semantics, although Stencila uses one type with an explicit `level` rather than separate `h1` to `h6` element types.
- JATS [`<title>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/title.html): Closest JATS analogue for section and container titles, though JATS heading semantics are tied to parent structures rather than a standalone heading node with explicit level.
- Pandoc [`Header`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Header): Close block analogue with an explicit level; Stencila also supports labeling, authorship, and provenance metadata on headings.
- MDAST [`Heading`](https://github.com/syntax-tree/mdast#heading): Closest MDAST block analogue for headings.

# Properties

The `Heading` type has these properties:

| Name         | Description                                                                     | Type                                        | Inherited from          |
| ------------ | ------------------------------------------------------------------------------- | ------------------------------------------- | ----------------------- |
| `labelType`  | The type of the label for the appendix (if present, should be `AppendixLabel`). | [`LabelType`](./label-type.md)              | -                       |
| `label`      | A short label for the heading.                                                  | [`String`](./string.md)                     | -                       |
| `level`      | The level of the heading.                                                       | [`Integer`](./integer.md)                   | -                       |
| `content`    | Content of the heading.                                                         | [`Inline`](./inline.md)*                    | -                       |
| `authors`    | The authors of the heading.                                                     | [`Author`](./author.md)*                    | -                       |
| `provenance` | A summary of the provenance of the content within the heading.                  | [`ProvenanceCount`](./provenance-count.md)* | -                       |
| `id`         | The identifier for this item.                                                   | [`String`](./string.md)                     | [`Entity`](./entity.md) |

# Related

The `Heading` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Heading` type is represented in:

- [JSON-LD](https://stencila.org/Heading.jsonld)
- [JSON Schema](https://stencila.org/Heading.schema.json)
- Python class [`Heading`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Heading`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/heading.rs)
- TypeScript class [`Heading`](https://github.com/stencila/stencila/blob/main/ts/src/types/Heading.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Heading` type are generated using the following strategies.

::: table

| Property    | Complexity | Description                                                                     | Strategy                                      |
| ----------- | ---------- | ------------------------------------------------------------------------------- | --------------------------------------------- |
| `labelType` | Min+       | No label type                                                                   | `None`                                        |
| `label`     | Min+       | No label                                                                        | `None`                                        |
| `level`     | Min+       | Fixed value of 1                                                                | `1`                                           |
|             | Low+       | Generate values between 1 and 6                                                 | `1..=6i64`                                    |
|             | High+      | Generate values between 0 and 6                                                 | `0..=6i64`                                    |
|             | Max        | Generate an arbitrary value                                                     | `i64::arbitrary()`                            |
| `content`   | Min+       | Generate a single arbitrary inline node                                         | `vec_inlines(1)`                              |
|             | Low+       | Generate up to two arbitrary inline nodes                                       | `vec_inlines(2)`                              |
|             | High+      | Generate up to four arbitrary inline nodes                                      | `vec_inlines(4)`                              |
|             | Max        | Generate up to eight arbitrary inline nodes without restrictions on their order | `vec(Inline::arbitrary(), size_range(0..=8))` |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`Heading.yaml`](https://github.com/stencila/stencila/blob/main/schema/Heading.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
