---
title: Citation Group
description: A group of `Citation` nodes.
---

This type allows you to group associated citations together.
When some content in a [`Creative Work`](./CreativeWork) Citations more than one
reference for a particular piece of text, use a `CitationGroup` to encapsulate
multiple [`Citation`](./Citation) nodes.

At present we do not give a `citationMode` property to a `CitationGroup` since
they will almost always be parenthetical as opposed to narrative.
In other words, it usually only makes sense for individual `Citation` nodes to be
narrative (although they may be connected together within `content` using words
such as "and").


# Properties

The `CitationGroup` type has these properties:

| Name      | Description                                                                 | Type                         | Inherited from          | `JSON-LD @id`                                                  | Aliases |
| --------- | --------------------------------------------------------------------------- | ---------------------------- | ----------------------- | -------------------------------------------------------------- | ------- |
| `id`      | The identifier for this item.                                               | [`String`](./string.md)      | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)                           | -       |
| `items`   | One or more `Citation`s to be referenced in the same surrounding text.      | [`Citation`](./citation.md)* | -                       | [`schema:itemListElement`](https://schema.org/itemListElement) | `item`  |
| `content` | A rendering of the citation group using the citation style of the document. | [`Inline`](./inline.md)*     | -                       | `stencila:content`                                             | -       |

# Related

The `CitationGroup` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `CitationGroup` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                            | Notes |
| ------------------------------------------------ | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                    |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |                                    |
| [JATS](../formats/jats.md)                       |              |              | Encoded using special function     |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              | Encoded using implemented function |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |                                    |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |                                    |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |                                    |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |                                    |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                    |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                    |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                    |
| [Citation File Format](../formats/cff.md)        |              |              |                                    |
| [CSV](../formats/csv.md)                         |              |              |                                    |
| [TSV](../formats/tsv.md)                         |              |              |                                    |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                    |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                    |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                    |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                    |
| [Directory](../formats/directory.md)             |              |              |                                    |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                    |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                    |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                    |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                    |
| [Email HTML](../formats/email.html.md)           |              |              |                                    |
| [MJML](../formats/mjml.md)                       |              |              |                                    |

# Bindings

The `CitationGroup` type is represented in:

- [JSON-LD](https://stencila.org/CitationGroup.jsonld)
- [JSON Schema](https://stencila.org/CitationGroup.schema.json)
- Python class [`CitationGroup`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/citation_group.py)
- Rust struct [`CitationGroup`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/citation_group.rs)
- TypeScript class [`CitationGroup`](https://github.com/stencila/stencila/blob/main/ts/src/types/CitationGroup.ts)

# Source

This documentation was generated from [`CitationGroup.yaml`](https://github.com/stencila/stencila/blob/main/schema/CitationGroup.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
