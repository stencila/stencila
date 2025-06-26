---
title: Styled Inline
description: Styled inline content.
config:
  publish:
    ghost:
      type: post
      slug: styled-inline
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Style
---

This type is marked as unstable and is subject to change.

# Properties

The `StyledInline` type has these properties:

| Name                  | Description                                                            | Type                                                                                         | Inherited from                                                     | `JSON-LD @id`                                | Aliases                                                                                                            |
| --------------------- | ---------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                                          | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)         | -                                                                                                                  |
| `code`                | The code of the equation in the `styleLanguage`.                       | [`Cord`](https://stencila.ghost.io/docs/reference/schema/cord)                               | [`Styled`](https://stencila.ghost.io/docs/reference/schema/styled) | `stencila:code`                              | -                                                                                                                  |
| `styleLanguage`       | The language used for the style specification e.g. css, tw             | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | [`Styled`](https://stencila.ghost.io/docs/reference/schema/styled) | `stencila:styleLanguage`                     | `style-language`, `style_language`                                                                                 |
| `authors`             | The authors of the code and content in the styled node.                | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                          | [`Styled`](https://stencila.ghost.io/docs/reference/schema/styled) | [`schema:author`](https://schema.org/author) | `author`                                                                                                           |
| `provenance`          | A summary of the provenance of the code and content in the styed node. | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)*       | [`Styled`](https://stencila.ghost.io/docs/reference/schema/styled) | `stencila:provenance`                        | -                                                                                                                  |
| `compilationDigest`   | A digest of the `code` and `styleLanguage`.                            | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)    | [`Styled`](https://stencila.ghost.io/docs/reference/schema/styled) | `stencila:compilationDigest`                 | `compilation-digest`, `compilation_digest`                                                                         |
| `compilationMessages` | Messages generated while parsing and transpiling the style.            | [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message)* | [`Styled`](https://stencila.ghost.io/docs/reference/schema/styled) | `stencila:compilationMessages`               | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |
| `css`                 | A Cascading Style Sheet (CSS) transpiled from the `code` property.     | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | [`Styled`](https://stencila.ghost.io/docs/reference/schema/styled) | `stencila:css`                               | -                                                                                                                  |
| `classList`           | A space separated list of class names associated with the node.        | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | [`Styled`](https://stencila.ghost.io/docs/reference/schema/styled) | `stencila:classList`                         | `class-list`, `class_list`                                                                                         |
| `content`             | The content within the span.                                           | [`Inline`](https://stencila.ghost.io/docs/reference/schema/inline)*                          | -                                                                  | `stencila:content`                           | -                                                                                                                  |

# Related

The `StyledInline` type is related to these types:

- Parents: [`Styled`](https://stencila.ghost.io/docs/reference/schema/styled)
- Children: none

# Formats

The `StyledInline` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding   | Support                                                                                                                | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ---------- | ---------------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |            |                                                                                                                        |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |            | Encoded as [`<span>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span)                                  |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       | 🟢 No loss    | 🟢 No loss  | Encoded as [`<styled-content>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/styled-content.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | ⚠️ High loss |            | Encoded using implemented function                                                                                     |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | ⚠️ High loss |            |                                                                                                                        |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | ⚠️ High loss |            |                                                                                                                        |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | ⚠️ High loss |            |                                                                                                                        |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | ⚠️ High loss |            |                                                                                                                        |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss |                                                                                                                        |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss |                                                                                                                        |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss |            |                                                                                                                        |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |            |                                                                                                                        |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss |                                                                                                                        |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss |                                                                                                                        |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |            |                                                                                                                        |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss |                                                                                                                        |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss |                                                                                                                        |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss  |                                                                                                                        |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss  |                                                                                                                        |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss  |                                                                                                                        |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss  |                                                                                                                        |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss  |                                                                                                                        |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss  |                                                                                                                        |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss  |                                                                                                                        |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss |                                                                                                                        |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss |                                                                                                                        |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss |                                                                                                                        |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |            |                                                                                                                        |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |            |                                                                                                                        |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |            |                                                                                                                        |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss |                                                                                                                        |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss |                                                                                                                        |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |            |                                                                                                                        |

# Bindings

The `StyledInline` type is represented in:

- [JSON-LD](https://stencila.org/StyledInline.jsonld)
- [JSON Schema](https://stencila.org/StyledInline.schema.json)
- Python class [`StyledInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/styled_inline.py)
- Rust struct [`StyledInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/styled_inline.rs)
- TypeScript class [`StyledInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/StyledInline.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `StyledInline` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property        | Complexity | Description                                                                                                                       | Strategy                                                                                                                                                                    |
| --------------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `code`          | Min+       | Generate a simple fixed string of code.                                                                                           | `Cord::from("code")`                                                                                                                                                        |
|                 | Low+       | Generate a random string of up to 10 alphanumeric & space characters (trimmed). Avoid keywords used to identify other node types. | `r"[a-zA-Z0-9 ]{1,10}".prop_filter("No keywords", \|code\| !["include", "call", "if", "ifblock", "for"].contains(&code.trim())).prop_map(\|code\| Cord::from(code.trim()))` |
|                 | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                  | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`                                                                                                                                   |
|                 | Max        | Generate an arbitrary string.                                                                                                     | `String::arbitrary().prop_map(Cord::from)`                                                                                                                                  |
| `styleLanguage` | Min+       | Do not generate a style language.                                                                                                 | `None`                                                                                                                                                                      |
|                 | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                     | `option::of(r"[a-zA-Z0-9]{1,10}")`                                                                                                                                          |
|                 | Max        | Generate an arbitrary string.                                                                                                     | `option::of(String::arbitrary())`                                                                                                                                           |
| `content`       | Min+       | Generate a single fixed text value.                                                                                               | `vec![t("text")]`                                                                                                                                                           |
|                 | High+      | Generate up to two arbitrary, non-recursive, inline nodes                                                                         | `vec_inlines_non_recursive(2)`                                                                                                                                              |
|                 | Max        | Generate up to four arbitrary, non-recursive, inline nodes                                                                        | `vec_inlines_non_recursive(4)`                                                                                                                                              |

# Source

This documentation was generated from [`StyledInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/StyledInline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
