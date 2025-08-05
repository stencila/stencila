---
title: Admonition
description: A admonition within a document.
config:
  publish:
    ghost:
      type: post
      slug: admonition
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

# Properties

The `Admonition` type has these properties:

| Name             | Description                                                       | Type                                                                                   | Inherited from                                                     | `JSON-LD @id`                                    | Aliases                              |
| ---------------- | ----------------------------------------------------------------- | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------ | ------------------------------------ |
| `id`             | The identifier for this item.                                     | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                     | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)             | -                                    |
| `admonitionType` | The type of admonition.                                           | [`AdmonitionType`](https://stencila.ghost.io/docs/reference/schema/admonition-type)    | -                                                                  | `stencila:admonitionType`                        | `admonition-type`, `admonition_type` |
| `title`          | The title of the admonition.                                      | [`Inline`](https://stencila.ghost.io/docs/reference/schema/inline)*                    | -                                                                  | [`schema:headline`](https://schema.org/headline) | -                                    |
| `isFolded`       | Whether the admonition is folded.                                 | [`Boolean`](https://stencila.ghost.io/docs/reference/schema/boolean)                   | -                                                                  | `stencila:isFolded`                              | `is-folded`, `is_folded`             |
| `content`        | The content within the section.                                   | [`Block`](https://stencila.ghost.io/docs/reference/schema/block)*                      | -                                                                  | `stencila:content`                               | -                                    |
| `authors`        | The authors of the admonition.                                    | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                    | -                                                                  | [`schema:author`](https://schema.org/author)     | `author`                             |
| `provenance`     | A summary of the provenance of the content within the admonition. | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)* | -                                                                  | `stencila:provenance`                            | -                                    |

# Related

The `Admonition` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `Admonition` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                                                                                                        | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | -------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                                                                                                |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              | Encoded as [`<aside>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside)                        |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       | 🟢 No loss    | 🟢 No loss    | Encoded as [`<boxed-text>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/boxed-text.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | 🟢 No loss    | 🟢 No loss    | Encoded using implemented function                                                                             |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                                |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                                                                                                |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |                                                                                                                |
| [OpenDocument Text](https://stencila.ghost.io/docs/reference/formats/odt)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                |
| [CSL-JSON](https://stencila.ghost.io/docs/reference/formats/csl)                    |              |              |                                                                                                                |
| [Citation File Format](https://stencila.ghost.io/docs/reference/formats/cff)        |              |              |                                                                                                                |
| [CSV](https://stencila.ghost.io/docs/reference/formats/csv)                         |              |              |                                                                                                                |
| [TSV](https://stencila.ghost.io/docs/reference/formats/tsv)                         |              |              |                                                                                                                |
| [Apache Parquet](https://stencila.ghost.io/docs/reference/formats/parquet)          |              |              |                                                                                                                |
| [Apache Arrow](https://stencila.ghost.io/docs/reference/formats/arrow)              |              |              |                                                                                                                |
| [Microsoft Excel XLSX](https://stencila.ghost.io/docs/reference/formats/xlsx)       |              |              |                                                                                                                |
| [Microsoft Excel XLS](https://stencila.ghost.io/docs/reference/formats/xls)         |              |              |                                                                                                                |
| [OpenDocument Spreadsheet](https://stencila.ghost.io/docs/reference/formats/ods)    |              |              |                                                                                                                |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                                                                                                |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                                                                                                |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                                                                                                |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                                                                                                |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |                                                                                                                |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                                                                                                |

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
