---
title: Styled
description: An abstract base class for a document node that has styling applied to it and/or its content.
config:
  publish:
    ghost:
      type: page
      slug: styled
      state: publish
      tags:
      - '#schema'
      - '#doc'
      - Style
---

This class is very similar to the `Math` abstract base class but has `styleLanguage` instead
of `mathLanguage` and compiled `css` instead of `mathml`.

Note also that `styleLanguage` is optional.


This type is marked as unstable and is subject to change.

## Properties

The `Styled` type has these properties:

| Name                  | Description                                                            | Type                                                                                         | Inherited from                                                     | `JSON-LD @id`                                | Aliases                                                                                                            |
| --------------------- | ---------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                                          | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)         | -                                                                                                                  |
| `code`                | The code of the equation in the `styleLanguage`.                       | [`Cord`](https://stencila.ghost.io/docs/reference/schema/cord)                               | -                                                                  | `stencila:code`                              | -                                                                                                                  |
| `styleLanguage`       | The language used for the style specification e.g. css, tw             | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | -                                                                  | `stencila:styleLanguage`                     | `style-language`, `style_language`                                                                                 |
| `authors`             | The authors of the code and content in the styled node.                | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                          | -                                                                  | [`schema:author`](https://schema.org/author) | `author`                                                                                                           |
| `provenance`          | A summary of the provenance of the code and content in the styed node. | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)*       | -                                                                  | `stencila:provenance`                        | -                                                                                                                  |
| `compilationDigest`   | A digest of the `code` and `styleLanguage`.                            | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)    | -                                                                  | `stencila:compilationDigest`                 | `compilation-digest`, `compilation_digest`                                                                         |
| `compilationMessages` | Messages generated while parsing and transpiling the style.            | [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message)* | -                                                                  | `stencila:compilationMessages`               | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |
| `css`                 | A Cascading Style Sheet (CSS) transpiled from the `code` property.     | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | -                                                                  | `stencila:css`                               | -                                                                                                                  |
| `classList`           | A space separated list of class names associated with the node.        | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | -                                                                  | `stencila:classList`                         | `class-list`, `class_list`                                                                                         |

## Related

The `Styled` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: [`StyledBlock`](https://stencila.ghost.io/docs/reference/schema/styled-block), [`StyledInline`](https://stencila.ghost.io/docs/reference/schema/styled-inline)

## Bindings

The `Styled` type is represented in:

- [JSON-LD](https://stencila.org/Styled.jsonld)
- [JSON Schema](https://stencila.org/Styled.schema.json)
- Python class [`Styled`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/styled.py)
- Rust struct [`Styled`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/styled.rs)
- TypeScript class [`Styled`](https://github.com/stencila/stencila/blob/main/ts/src/types/Styled.ts)

## Source

This documentation was generated from [`Styled.yaml`](https://github.com/stencila/stencila/blob/main/schema/Styled.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
