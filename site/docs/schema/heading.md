---
title: Heading
description: A heading.
---

Analogues of `Heading` in other schemas include:
  - HTML [`<h1>` to `<h6>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h1)
  - JATS XML [`<title>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/title.html)
  - Pandoc [`Header`](https://github.com/jgm/pandoc-types/blob/1.17.5.4/Text/Pandoc/Definition.hs#L233)


# Properties

The `Heading` type has these properties:

| Name         | Description                                                                     | Type                                        | Inherited from          | `JSON-LD @id`                                | Aliases                    |
| ------------ | ------------------------------------------------------------------------------- | ------------------------------------------- | ----------------------- | -------------------------------------------- | -------------------------- |
| `id`         | The identifier for this item.                                                   | [`String`](./string.md)                     | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)         | -                          |
| `labelType`  | The type of the label for the appendix (if present, should be `AppendixLabel`). | [`LabelType`](./label-type.md)              | -                       | `stencila:labelType`                         | `label-type`, `label_type` |
| `label`      | A short label for the heading.                                                  | [`String`](./string.md)                     | -                       | `stencila:label`                             | -                          |
| `level`      | The level of the heading.                                                       | [`Integer`](./integer.md)                   | -                       | `stencila:level`                             | -                          |
| `content`    | Content of the heading.                                                         | [`Inline`](./inline.md)*                    | -                       | `stencila:content`                           | -                          |
| `authors`    | The authors of the heading.                                                     | [`Author`](./author.md)*                    | -                       | [`schema:author`](https://schema.org/author) | `author`                   |
| `provenance` | A summary of the provenance of the content within the heading.                  | [`ProvenanceCount`](./provenance-count.md)* | -                       | `stencila:provenance`                        | -                          |

# Related

The `Heading` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `Heading` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                                                     | Notes |
| ------------------------------------------------ | ------------ | ------------ | --------------------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                                             |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              | Encoded using special function                                                                                              |
| [JATS](../formats/jats.md)                       | 🟢 No loss    | 🟢 No loss    | Encoded as [`<title>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/title.html) using special function |
| [Markdown](../formats/md.md)                     | 🟢 No loss    | 🟢 No loss    | Encoded using implemented function                                                                                          |
| [Stencila Markdown](../formats/smd.md)           | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [Quarto Markdown](../formats/qmd.md)             | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [MyST Markdown](../formats/myst.md)              | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [LLM Markdown](../formats/llmd.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                                             |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                                                             |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                             |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                             |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                                                             |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                                                             |
| [CSV](../formats/csv.md)                         |              |              |                                                                                                                             |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                                                             |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                                                             |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                                                             |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                                                             |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                                                             |
| [Directory](../formats/directory.md)             |              |              |                                                                                                                             |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                                                             |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                                                             |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                                                             |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                                                             |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                                                             |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                                                             |

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
