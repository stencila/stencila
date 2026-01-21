---
title: Admonition
description: An admonition within a document.
---

Highlight important ideas or signal content that’s supplemental or only relevant in certain situations.


# Properties

The `Admonition` type has these properties:

| Name             | Description                                                       | Type                                        | Inherited from          |
| ---------------- | ----------------------------------------------------------------- | ------------------------------------------- | ----------------------- |
| `id`             | The identifier for this item.                                     | [`String`](./string.md)                     | [`Entity`](./entity.md) |
| `admonitionType` | The type of admonition.                                           | [`AdmonitionType`](./admonition-type.md)    | -                       |
| `title`          | The title of the admonition.                                      | [`Inline`](./inline.md)*                    | -                       |
| `isFolded`       | Whether the admonition is folded.                                 | [`Boolean`](./boolean.md)                   | -                       |
| `content`        | The content within the section.                                   | [`Block`](./block.md)*                      | -                       |
| `authors`        | The authors of the admonition.                                    | [`Author`](./author.md)*                    | -                       |
| `provenance`     | A summary of the provenance of the content within the admonition. | [`ProvenanceCount`](./provenance-count.md)* | -                       |

# Related

The `Admonition` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Admonition` type is represented in:

- [JSON-LD](https://stencila.org/Admonition.jsonld)
- [JSON Schema](https://stencila.org/Admonition.schema.json)
- Python class [`Admonition`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/admonition.py)
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

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`Admonition.yaml`](https://github.com/stencila/stencila/blob/main/schema/Admonition.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
