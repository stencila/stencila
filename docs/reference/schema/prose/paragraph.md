---
title: Paragraph
description: A paragraph.
config:
  publish:
    ghost:
      type: post
      slug: paragraph
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

Analogues of `Paragraph` in other schema include:
  - HTML [`<p>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p)
  - JATS XML [`<p>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/p.html)
  - MDAST [`Paragraph`](https://github.com/syntax-tree/mdast#Paragraph)
  - OpenDocument [`<text:p>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415138_253892949)
  - Pandoc [`Para`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L220)


# Properties

The `Paragraph` type has these properties:

| Name         | Description                                                  | Type                                                                                   | Inherited from                                                     | `JSON-LD @id`                                | Aliases  |
| ------------ | ------------------------------------------------------------ | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------- | -------- |
| `id`         | The identifier for this item.                                | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                     | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)         | -        |
| `content`    | The contents of the paragraph.                               | [`Inline`](https://stencila.ghost.io/docs/reference/schema/inline)*                    | -                                                                  | `stencila:content`                           | -        |
| `authors`    | The authors of the paragraph.                                | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                    | -                                                                  | [`schema:author`](https://schema.org/author) | `author` |
| `provenance` | A summary of the provenance of content within the paragraph. | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)* | -                                                                  | `stencila:provenance`                        | -        |

# Related

The `Paragraph` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `Paragraph` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                                                                                      | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | -------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                                                                              |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              | Encoded as [`<p>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p)              |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       | 🟢 No loss    | 🟢 No loss    | Encoded as [`<p>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/p.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | 🟢 No loss    | 🟢 No loss    | Encoded as `{{content}}\n\n`                                                                 |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                                                                              |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                                                                              |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |                                                                                              |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                                                                              |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |                                                                                              |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                                                                              |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                                                                              |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                                                                              |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                                                                              |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |                                                                                              |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                                                                              |

# Bindings

The `Paragraph` type is represented in:

- [JSON-LD](https://stencila.org/Paragraph.jsonld)
- [JSON Schema](https://stencila.org/Paragraph.schema.json)
- Python class [`Paragraph`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/paragraph.py)
- Rust struct [`Paragraph`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/paragraph.rs)
- TypeScript class [`Paragraph`](https://github.com/stencila/stencila/blob/main/ts/src/types/Paragraph.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Paragraph` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                                     | Strategy                                      |
| --------- | ---------- | ------------------------------------------------------------------------------- | --------------------------------------------- |
| `content` | Min+       | Generate a single arbitrary inline node                                         | `vec_inlines(1)`                              |
|           | Low+       | Generate up to two arbitrary inline nodes                                       | `vec_inlines(2)`                              |
|           | High+      | Generate up to four arbitrary inline nodes                                      | `vec_inlines(4)`                              |
|           | Max        | Generate up to eight arbitrary inline nodes without restrictions on their order | `vec(Inline::arbitrary(), size_range(0..=8))` |

# Source

This documentation was generated from [`Paragraph.yaml`](https://github.com/stencila/stencila/blob/main/schema/Paragraph.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
