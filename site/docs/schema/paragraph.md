---
title: Paragraph
description: A paragraph.
---

Analogues of `Paragraph` in other schema include:
  - HTML [`<p>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p)
  - JATS XML [`<p>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/p.html)
  - MDAST [`Paragraph`](https://github.com/syntax-tree/mdast#Paragraph)
  - OpenDocument [`<text:p>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415138_253892949)
  - Pandoc [`Para`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L220)


# Properties

The `Paragraph` type has these properties:

| Name         | Description                                                  | Type                                        | Inherited from          | `JSON-LD @id`                                | Aliases  |
| ------------ | ------------------------------------------------------------ | ------------------------------------------- | ----------------------- | -------------------------------------------- | -------- |
| `id`         | The identifier for this item.                                | [`String`](./string.md)                     | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)         | -        |
| `content`    | The contents of the paragraph.                               | [`Inline`](./inline.md)*                    | -                       | `stencila:content`                           | -        |
| `authors`    | The authors of the paragraph.                                | [`Author`](./author.md)*                    | -                       | [`schema:author`](https://schema.org/author) | `author` |
| `provenance` | A summary of the provenance of content within the paragraph. | [`ProvenanceCount`](./provenance-count.md)* | -                       | `stencila:provenance`                        | -        |

# Related

The `Paragraph` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `Paragraph` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                      | Notes |
| ------------------------------------------------ | ------------ | ------------ | -------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                              |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              | Encoded as [`<p>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p)              |
| [JATS](../formats/jats.md)                       | 🟢 No loss    | 🟢 No loss    | Encoded as [`<p>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/p.html) |
| [Markdown](../formats/md.md)                     | 🟢 No loss    | 🟢 No loss    | Encoded using implemented function                                                           |
| [Stencila Markdown](../formats/smd.md)           | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [Quarto Markdown](../formats/qmd.md)             | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [MyST Markdown](../formats/myst.md)              | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [LLM Markdown](../formats/llmd.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                              |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                              |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                              |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                              |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                              |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                              |
| [CSV](../formats/csv.md)                         |              |              |                                                                                              |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                              |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                              |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                              |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                              |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                              |
| [Directory](../formats/directory.md)             |              |              |                                                                                              |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                              |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                              |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                              |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                              |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                              |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                              |

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
