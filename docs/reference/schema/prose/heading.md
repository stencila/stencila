---
title: Heading
description: A heading.
config:
  publish:
    ghost:
      type: post
      slug: heading
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

Analogues of `Heading` in other schemas include:
  - HTML [`<h1>` to `<h6>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h1)
  - JATS XML [`<title>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/title.html)
  - Pandoc [`Header`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L233)


# Properties

The `Heading` type has these properties:

| Name         | Description                                                                     | Type                                                                                   | Inherited from                                                     | `JSON-LD @id`                                | Aliases                    |
| ------------ | ------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------- | -------------------------- |
| `id`         | The identifier for this item.                                                   | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                     | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)         | -                          |
| `labelType`  | The type of the label for the appendix (if present, should be `AppendixLabel`). | [`LabelType`](https://stencila.ghost.io/docs/reference/schema/label-type)              | -                                                                  | `stencila:labelType`                         | `label-type`, `label_type` |
| `label`      | A short label for the heading.                                                  | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                     | -                                                                  | `stencila:label`                             | -                          |
| `level`      | The level of the heading.                                                       | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)                   | -                                                                  | `stencila:level`                             | -                          |
| `content`    | Content of the heading.                                                         | [`Inline`](https://stencila.ghost.io/docs/reference/schema/inline)*                    | -                                                                  | `stencila:content`                           | -                          |
| `authors`    | The authors of the heading.                                                     | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                    | -                                                                  | [`schema:author`](https://schema.org/author) | `author`                   |
| `provenance` | A summary of the provenance of the content within the heading.                  | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)* | -                                                                  | `stencila:provenance`                        | -                          |

# Related

The `Heading` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `Heading` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                                                                                                                     | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | --------------------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                                                                                                             |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              | Encoded using special function                                                                                              |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       | 🟢 No loss    | 🟢 No loss    | Encoded as [`<title>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/title.html) using special function |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | 🟢 No loss    | 🟢 No loss    | Encoded using implemented function                                                                                          |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                                             |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                                                                                                             |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |                                                                                                                             |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                                                                                                             |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |                                                                                                                             |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                                                                                                             |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                                                                                                             |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                                                                                                             |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                                                                                                             |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |                                                                                                                             |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                                                                                                             |

# Bindings

The `Heading` type is represented in:

- [JSON-LD](https://stencila.org/Heading.jsonld)
- [JSON Schema](https://stencila.org/Heading.schema.json)
- Python class [`Heading`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/heading.py)
- Rust struct [`Heading`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/heading.rs)
- TypeScript class [`Heading`](https://github.com/stencila/stencila/blob/main/ts/src/types/Heading.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Heading` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

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

# Source

This documentation was generated from [`Heading.yaml`](https://github.com/stencila/stencila/blob/main/schema/Heading.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
