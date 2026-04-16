---
title: Styled
description: An abstract base type for styled content.
---

This is an abstract base type used in Stencila Schema for nodes with explicit style
code.

It exists to represent styling separately from a node's core structural or
semantic role, while still allowing style code, style language, authorship,
provenance, and compiled CSS to be stored within the document model. This
supports authoring, theming, and style-aware publishing workflows.

Key properties include `code`, `styleLanguage`, `compilationDigest`, `css`,
and `classList`.


This type is marked as unstable and is subject to change.

# Properties

The `Styled` type has these properties:

| Name                  | Description                                                            | Type                                              | Inherited from          |
| --------------------- | ---------------------------------------------------------------------- | ------------------------------------------------- | ----------------------- |
| `code`                | The code of the equation in the `styleLanguage`.                       | [`Cord`](./cord.md)                               | -                       |
| `styleLanguage`       | The language used for the style specification e.g. css, tw             | [`String`](./string.md)                           | -                       |
| `authors`             | The authors of the code and content in the styled node.                | [`Author`](./author.md)*                          | -                       |
| `provenance`          | A summary of the provenance of the code and content in the styed node. | [`ProvenanceCount`](./provenance-count.md)*       | -                       |
| `compilationDigest`   | A digest of the `code` and `styleLanguage`.                            | [`CompilationDigest`](./compilation-digest.md)    | -                       |
| `compilationMessages` | Messages generated while parsing and transpiling the style.            | [`CompilationMessage`](./compilation-message.md)* | -                       |
| `css`                 | A Cascading Style Sheet (CSS) transpiled from the `code` property.     | [`String`](./string.md)                           | -                       |
| `classList`           | A space separated list of class names associated with the node.        | [`String`](./string.md)                           | -                       |
| `id`                  | The identifier for this item.                                          | [`String`](./string.md)                           | [`Entity`](./entity.md) |

# Related

The `Styled` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: [`StyledBlock`](./styled-block.md), [`StyledInline`](./styled-inline.md)

# Bindings

The `Styled` type is represented in:

- [JSON-LD](https://stencila.org/Styled.jsonld)
- [JSON Schema](https://stencila.org/Styled.schema.json)
- Python class [`Styled`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Styled`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/styled.rs)
- TypeScript class [`Styled`](https://github.com/stencila/stencila/blob/main/ts/src/types/Styled.ts)

***

This documentation was generated from [`Styled.yaml`](https://github.com/stencila/stencila/blob/main/schema/Styled.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
