---
title: Enumeration
description: Lists or enumerations, for example, a list of cuisines or music genres, etc.
---

# Properties

The `Enumeration` type has these properties:

| Name             | Description                                   | Type                                                                 | Inherited from          | `JSON-LD @id`                                              | Aliases                                                                                   |
| ---------------- | --------------------------------------------- | -------------------------------------------------------------------- | ----------------------- | ---------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `id`             | The identifier for this item.                 | [`String`](./string.md)                                              | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)                       | -                                                                                         |
| `alternateNames` | Alternate names (aliases) for the item.       | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   | [`schema:alternateName`](https://schema.org/alternateName) | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` |
| `description`    | A description of the item.                    | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:description`](https://schema.org/description)     | -                                                                                         |
| `identifiers`    | Any kind of identifier for any kind of Thing. | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   | [`schema:identifier`](https://schema.org/identifier)       | `identifier`                                                                              |
| `images`         | Images of the item.                           | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   | [`schema:image`](https://schema.org/image)                 | `image`                                                                                   |
| `name`           | The name of the item.                         | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:name`](https://schema.org/name)                   | -                                                                                         |
| `url`            | The URL of the item.                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:url`](https://schema.org/url)                     | -                                                                                         |

# Related

The `Enumeration` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: [`AdmonitionType`](./admonition-type.md), [`AuthorRoleName`](./author-role-name.md), [`CitationIntent`](./citation-intent.md), [`CitationMode`](./citation-mode.md), [`ClaimType`](./claim-type.md), [`ConfigPublishGhostState`](./config-publish-ghost-state.md), [`ConfigPublishGhostType`](./config-publish-ghost-type.md), [`ConfigPublishZenodoAccessRight`](./config-publish-zenodo-access-right.md), [`CreativeWorkType`](./creative-work-type.md), [`ExecutionBounds`](./execution-bounds.md), [`ExecutionDependantRelation`](./execution-dependant-relation.md), [`ExecutionDependencyRelation`](./execution-dependency-relation.md), [`ExecutionMode`](./execution-mode.md), [`ExecutionRequired`](./execution-required.md), [`ExecutionStatus`](./execution-status.md), [`FormDeriveAction`](./form-derive-action.md), [`HorizontalAlignment`](./horizontal-alignment.md), [`InstructionType`](./instruction-type.md), [`LabelType`](./label-type.md), [`ListOrder`](./list-order.md), [`MessageLevel`](./message-level.md), [`MessageRole`](./message-role.md), [`NoteType`](./note-type.md), [`ProvenanceCategory`](./provenance-category.md), [`RelativePosition`](./relative-position.md), [`SectionType`](./section-type.md), [`SuggestionStatus`](./suggestion-status.md), [`TableCellType`](./table-cell-type.md), [`TableRowType`](./table-row-type.md), [`TimeUnit`](./time-unit.md), [`VerticalAlignment`](./vertical-alignment.md)

# Formats

The `Enumeration` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support | Notes |
| ------------------------------------------------ | ------------ | ------------ | ------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |         |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |         |
| [JATS](../formats/jats.md)                       |              |              |         |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              |         |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |         |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |         |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |         |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |         |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |         |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |         |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |         |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |         |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |         |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |         |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |         |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |         |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |         |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |         |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |         |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |         |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |         |
| [CSL-JSON](../formats/csl.md)                    |              |              |         |
| [Citation File Format](../formats/cff.md)        |              |              |         |
| [CSV](../formats/csv.md)                         |              |              |         |
| [TSV](../formats/tsv.md)                         |              |              |         |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |         |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |         |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |         |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |         |
| [Directory](../formats/directory.md)             |              |              |         |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |         |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |         |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |         |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |         |
| [Email HTML](../formats/email.html.md)           |              |              |         |
| [MJML](../formats/mjml.md)                       |              |              |         |

# Bindings

The `Enumeration` type is represented in:

- [JSON-LD](https://stencila.org/Enumeration.jsonld)
- [JSON Schema](https://stencila.org/Enumeration.schema.json)
- Python class [`Enumeration`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/enumeration.py)
- Rust struct [`Enumeration`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/enumeration.rs)
- TypeScript class [`Enumeration`](https://github.com/stencila/stencila/blob/main/ts/src/types/Enumeration.ts)

# Source

This documentation was generated from [`Enumeration.yaml`](https://github.com/stencila/stencila/blob/main/schema/Enumeration.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
