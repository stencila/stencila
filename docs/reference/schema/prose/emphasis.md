---
title: Emphasis
description: Emphasized content.
config:
  publish:
    ghost:
      type: post
      slug: emphasis
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

Analogues of `Emphasis` in other schema include:
  - HTML [`<em>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em)
  - JATS XML [`<italic>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/italic.html)
  - MDAST [`Emphasis`](https://github.com/syntax-tree/mdast#emphasis)
  - Pandoc [`Emph`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L256)


# Properties

The `Emphasis` type has these properties:

| Name      | Description                   | Type                                                                | Inherited from                                                     | `JSON-LD @id`                        | Aliases |
| --------- | ----------------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------ | ------- |
| `id`      | The identifier for this item. | [`String`](https://stencila.ghost.io/docs/reference/schema/string)  | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -       |
| `content` | The content that is marked.   | [`Inline`](https://stencila.ghost.io/docs/reference/schema/inline)* | [`Mark`](https://stencila.ghost.io/docs/reference/schema/mark)     | `stencila:content`                   | -       |

# Related

The `Emphasis` type is related to these types:

- Parents: [`Mark`](https://stencila.ghost.io/docs/reference/schema/mark)
- Children: none

# Formats

The `Emphasis` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                                                                                                | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | ------------------------------------------------------------------------------------------------------ | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                                                                                        |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              | Encoded as [`<em>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em)                      |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       | 🟢 No loss    | 🟢 No loss    | Encoded as [`<italic>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/italic.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | 🟢 No loss    | 🟢 No loss    | Encoded as `_{{content}}_`                                                                             |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | 🟢 No loss    | 🟢 No loss    |                                                                                                        |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | 🟢 No loss    | 🟢 No loss    |                                                                                                        |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | 🟢 No loss    | 🟢 No loss    |                                                                                                        |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | 🟢 No loss    | 🟢 No loss    |                                                                                                        |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                        |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                        |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                        |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                                                                                        |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                        |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |                                                                                                        |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |                                                                                                        |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                        |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                        |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                        |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                                                                                        |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                        |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                        |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                        |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |                                                                                                        |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                        |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                        |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                        |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                        |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                                                                                        |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                                                                                        |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                                                                                        |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                                                                                        |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                                                                                        |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |                                                                                                        |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                                                                                        |

# Bindings

The `Emphasis` type is represented in:

- [JSON-LD](https://stencila.org/Emphasis.jsonld)
- [JSON Schema](https://stencila.org/Emphasis.schema.json)
- Python class [`Emphasis`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/emphasis.py)
- Rust struct [`Emphasis`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/emphasis.rs)
- TypeScript class [`Emphasis`](https://github.com/stencila/stencila/blob/main/ts/src/types/Emphasis.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Emphasis` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                | Strategy                       |
| --------- | ---------- | ---------------------------------------------------------- | ------------------------------ |
| `content` | Min+       | Generate a single fixed text value.                        | `vec![t("text")]`              |
|           | Low+       | Generate a single arbitrary, non-recursive, inline node    | `vec_inlines_non_recursive(1)` |
|           | High+      | Generate up to two arbitrary, non-recursive, inline nodes  | `vec_inlines_non_recursive(2)` |
|           | Max        | Generate up to four arbitrary, non-recursive, inline nodes | `vec_inlines_non_recursive(4)` |

# Source

This documentation was generated from [`Emphasis.yaml`](https://github.com/stencila/stencila/blob/main/schema/Emphasis.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
