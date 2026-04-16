---
title: Admonition
description: An admonition within a document.
---

This is a type used in Stencila Schema for admonition or callout blocks.

It exists to represent semantically distinct callouts such as notes,
warnings, tips, and cautions as structured document content rather than as
presentation-only containers. This supports consistent rendering, folding, and
transformation across authoring and publishing formats.

Key properties include `admonitionType`, `title`, `isFolded`, and `content`.


# Analogues

The following external types, elements, or nodes are similar to a `Admonition`:

- HTML [`<aside>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside): Closest HTML sectioning analogue, though HTML `<aside>` is broader and does not itself encode admonition type or folding behavior.
- JATS [`<boxed-text>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/boxed-text.html): Closest JATS analogue for boxed callout content, but JATS does not standardize the same admonition type vocabulary.
- Pandoc [`Div`](https://hackage-content.haskell.org/package/pandoc-types-1.23.1.1/docs/Text-Pandoc-Definition.html#v:Div): Pandoc admonitions are usually represented as divisions with classes and attributes rather than a dedicated node type.
- MyST directive [`admonition`](https://mystmd.org/guide/directives#directive-admonition)

# Properties

The `Admonition` type has these properties:

| Name             | Description                                                       | Type                                        | Inherited from          |
| ---------------- | ----------------------------------------------------------------- | ------------------------------------------- | ----------------------- |
| `admonitionType` | The type of admonition.                                           | [`AdmonitionType`](./admonition-type.md)    | -                       |
| `title`          | The title of the admonition.                                      | [`Inline`](./inline.md)*                    | -                       |
| `isFolded`       | Whether the admonition is folded.                                 | [`Boolean`](./boolean.md)                   | -                       |
| `content`        | The content within the section.                                   | [`Block`](./block.md)*                      | -                       |
| `authors`        | The authors of the admonition.                                    | [`Author`](./author.md)*                    | -                       |
| `provenance`     | A summary of the provenance of the content within the admonition. | [`ProvenanceCount`](./provenance-count.md)* | -                       |
| `id`             | The identifier for this item.                                     | [`String`](./string.md)                     | [`Entity`](./entity.md) |

# Related

The `Admonition` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Admonition` type is represented in:

- [JSON-LD](https://stencila.org/Admonition.jsonld)
- [JSON Schema](https://stencila.org/Admonition.schema.json)
- Python class [`Admonition`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Admonition`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/admonition.rs)
- TypeScript class [`Admonition`](https://github.com/stencila/stencila/blob/main/ts/src/types/Admonition.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Admonition` type are generated using the following strategies.

::: table

| Property         | Complexity | Description                                                 | Strategy                                   |
| ---------------- | ---------- | ----------------------------------------------------------- | ------------------------------------------ |
| `admonitionType` | Min+       | Fixed admonition type.                                      | `AdmonitionType::Info`                     |
|                  | Low+       | Generate an arbitrary admonition type.                      | `AdmonitionType::arbitrary()`              |
| `title`          | Min+       | No title.                                                   | `None`                                     |
|                  | Low+       | Generate up to two arbitrary, non-recursive, inline nodes.  | `option::of(vec_inlines_non_recursive(2))` |
|                  | High+      | Generate up to four arbitrary, non-recursive, inline nodes. | `option::of(vec_inlines_non_recursive(4))` |
| `isFolded`       | Min+       | Not foldable.                                               | `None`                                     |
|                  | Low+       | Arbitrarily, un-foldable, folded, or unfolded.              | `option::of(bool::arbitrary())`            |
| `content`        | Min+       | A single, simple paragraph.                                 | `vec![p([t("Admonition content")])]`       |
|                  | Low+       | Generate up to two arbitrary paragraphs.                    | `vec_paragraphs(2)`                        |
|                  | High+      | Generate up to four arbitrary, non-recursive, block nodes.  | `vec_blocks_non_recursive(4)`              |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`Admonition.yaml`](https://github.com/stencila/stencila/blob/main/schema/Admonition.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
