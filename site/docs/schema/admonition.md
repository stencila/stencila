---
title: Admonition
description: An admonition within a document.
---

Highlight important ideas or signal content that’s supplemental or only relevant in certain situations.


# Properties

The `Admonition` type has these properties:

| Name             | Description                                                       | Type                                        | Inherited from          | `JSON-LD @id`                                    | Aliases                              |
| ---------------- | ----------------------------------------------------------------- | ------------------------------------------- | ----------------------- | ------------------------------------------------ | ------------------------------------ |
| `id`             | The identifier for this item.                                     | [`String`](./string.md)                     | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)             | -                                    |
| `admonitionType` | The type of admonition.                                           | [`AdmonitionType`](./admonition-type.md)    | -                       | `stencila:admonitionType`                        | `admonition-type`, `admonition_type` |
| `title`          | The title of the admonition.                                      | [`Inline`](./inline.md)*                    | -                       | [`schema:headline`](https://schema.org/headline) | -                                    |
| `isFolded`       | Whether the admonition is folded.                                 | [`Boolean`](./boolean.md)                   | -                       | `stencila:isFolded`                              | `is-folded`, `is_folded`             |
| `content`        | The content within the section.                                   | [`Block`](./block.md)*                      | -                       | `stencila:content`                               | -                                    |
| `authors`        | The authors of the admonition.                                    | [`Author`](./author.md)*                    | -                       | [`schema:author`](https://schema.org/author)     | `author`                             |
| `provenance`     | A summary of the provenance of the content within the admonition. | [`ProvenanceCount`](./provenance-count.md)* | -                       | `stencila:provenance`                            | -                                    |

# Related

The `Admonition` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `Admonition` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                                        | Notes |
| ------------------------------------------------ | ------------ | ------------ | -------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                                |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              | Encoded as [`<aside>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside)                        |
| [JATS](../formats/jats.md)                       | 🟢 No loss    | 🟢 No loss    | Encoded as [`<boxed-text>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/boxed-text.html) |
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

The `Admonition` type is represented in:

- [JSON-LD](https://stencila.org/Admonition.jsonld)
- [JSON Schema](https://stencila.org/Admonition.schema.json)
- Python class [`Admonition`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/admonition.py)
- Rust struct [`Admonition`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/admonition.rs)
- TypeScript class [`Admonition`](https://github.com/stencila/stencila/blob/main/ts/src/types/Admonition.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Admonition` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property         | Complexity | Description                                                 | Strategy                                   |
| ---------------- | ---------- | ----------------------------------------------------------- | ------------------------------------------ |
| `admonitionType` | Min+       | Fixed admonition type.                                      | `AdmonitionType::Info`                     |
|                  | Low+       | Generate an arbitrary admonition type.                      | `AdmonitionType::arbitrary()`              |
| `title`          | Min+       | No title.                                                   | `None`                                     |
|                  | Low+       | Generate up to two arbitrary, non-recursive, inline nodes.  | `option::of(vec_inlines_non_recursive(2))` |
|                  | High+      | Generate up to four arbitrary, non-recursive, inline nodes. | `option::of(vec_inlines_non_recursive(4))` |
| `isFolded`       | Min+       | Not foldable.                                               | `None`                                     |
|                  | Low+       | Arbitrarily, un-foldable, folded, or unfolded.              | `option::of(bool::arbitrary())`            |
| `content`        | Min+       | A single, simple paragraph.                                 | `vec![p([t("Admonition content")])]`       |
|                  | Low+       | Generate up to two arbitrary paragraphs.                    | `vec_paragraphs(2)`                        |
|                  | High+      | Generate up to four arbitrary, non-recursive, block nodes.  | `vec_blocks_non_recursive(4)`              |

# Source

This documentation was generated from [`Admonition.yaml`](https://github.com/stencila/stencila/blob/main/schema/Admonition.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
