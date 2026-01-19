---
title: Quote Block
description: A section quoted from somewhere else.
---

# Properties

The `QuoteBlock` type has these properties:

| Name         | Description                                                    | Type                                               | Inherited from          | `JSON-LD @id`                                | Aliases  |
| ------------ | -------------------------------------------------------------- | -------------------------------------------------- | ----------------------- | -------------------------------------------- | -------- |
| `id`         | The identifier for this item.                                  | [`String`](./string.md)                            | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)         | -        |
| `source`     | The source of the quote.                                       | [`Citation`](./citation.md) \| [`Text`](./text.md) | -                       | `stencila:source`                            | -        |
| `content`    | The content of the quote.                                      | [`Block`](./block.md)*                             | -                       | `stencila:content`                           | -        |
| `authors`    | The authors of the quote.                                      | [`Author`](./author.md)*                           | -                       | [`schema:author`](https://schema.org/author) | `author` |
| `provenance` | A summary of the provenance of the content within the section. | [`ProvenanceCount`](./provenance-count.md)*        | -                       | `stencila:provenance`                        | -        |

# Related

The `QuoteBlock` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `QuoteBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                                        | Notes |
| ------------------------------------------------ | ------------ | ------------ | -------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                                |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              | Encoded as [`<blockquote>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/blockquote)              |
| [JATS](../formats/jats.md)                       | 🟢 No loss    | 🟢 No loss    | Encoded as [`<disp-quote>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/disp-quote.html) |
| [Markdown](../formats/md.md)                     | 🟢 No loss    | 🟢 No loss    | Encoded using implemented function                                                                             |
| [Stencila Markdown](../formats/smd.md)           | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [Quarto Markdown](../formats/qmd.md)             | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [MyST Markdown](../formats/myst.md)              | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [LLM Markdown](../formats/llmd.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                                |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                                                |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                                                |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                                                |
| [CSV](../formats/csv.md)                         |              |              |                                                                                                                |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                                                |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                                                |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                                                |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                                                |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                                                |
| [Directory](../formats/directory.md)             |              |              |                                                                                                                |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                                                |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                                                |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                                                |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                                                |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                                                |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                                                |

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
