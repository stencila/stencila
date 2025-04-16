---
title: Quote Block
description: A section quoted from somewhere else.
config:
  publish:
    ghost:
      type: post
      slug: quote-block
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

# Properties

The `QuoteBlock` type has these properties:

| Name         | Description                                                    | Type                                                                                                                                     | Inherited from                                                     | `JSON-LD @id`                                | Aliases  |
| ------------ | -------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------- | -------- |
| `id`         | The identifier for this item.                                  | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                       | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)         | -        |
| `source`     | The source of the quote.                                       | [`Citation`](https://stencila.ghost.io/docs/reference/schema/citation) \| [`Text`](https://stencila.ghost.io/docs/reference/schema/text) | -                                                                  | `stencila:source`                            | -        |
| `content`    | The content of the quote.                                      | [`Block`](https://stencila.ghost.io/docs/reference/schema/block)*                                                                        | -                                                                  | `stencila:content`                           | -        |
| `authors`    | The authors of the quote.                                      | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                                                                      | -                                                                  | [`schema:author`](https://schema.org/author) | `author` |
| `provenance` | A summary of the provenance of the content within the section. | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)*                                                   | -                                                                  | `stencila:provenance`                        | -        |

# Related

The `QuoteBlock` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `QuoteBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                       | Encoding     | Decoding   | Support                                                                                                        | Notes |
| ---------------------------------------------------------------------------- | ------------ | ---------- | -------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)        | 🟢 No loss    |            |                                                                                                                |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                | 🔷 Low loss   |            | Encoded as [`<blockquote>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/blockquote)              |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                | 🟢 No loss    | 🟢 No loss  | Encoded as [`<disp-quote>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/disp-quote.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)              | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function                                                                             |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)    | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)      | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)       | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)        | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)              | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                  | 🔷 Low loss   |            |                                                                                                                |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)          | ⚠️ High loss |            |                                                                                                                |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)              | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx) | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)     | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                  | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)        | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)              | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)           | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)      | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                | 🟢 No loss    | 🟢 No loss  |                                                                                                                |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)     | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)       | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)        | 🔷 Low loss   | 🔷 Low loss |                                                                                                                |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)      |              |            |                                                                                                                |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)  |              |            |                                                                                                                |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)              | 🔷 Low loss   |            |                                                                                                                |

# Bindings

The `QuoteBlock` type is represented in:

- [JSON-LD](https://stencila.org/QuoteBlock.jsonld)
- [JSON Schema](https://stencila.org/QuoteBlock.schema.json)
- Python class [`QuoteBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/quote_block.py)
- Rust struct [`QuoteBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/quote_block.rs)
- TypeScript class [`QuoteBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/QuoteBlock.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `QuoteBlock` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                 | Strategy                      |
| --------- | ---------- | ----------------------------------------------------------- | ----------------------------- |
| `content` | Min+       | Generate a single arbitrary paragraph.                      | `vec_paragraphs(1)`           |
|           | Low+       | Generate up to two arbitrary, non-recursive, block nodes.   | `vec_blocks_non_recursive(2)` |
|           | High+      | Generate up to four arbitrary, non-recursive, block nodes.  | `vec_blocks_non_recursive(4)` |
|           | Max        | Generate up to eight arbitrary, non-recursive, block nodes. | `vec_blocks_non_recursive(8)` |

# Source

This documentation was generated from [`QuoteBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/QuoteBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
