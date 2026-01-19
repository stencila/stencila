---
title: Link
description: A hyperlink to other pages, sections within the same document, resources, or any URL.
---

# Properties

The `Link` type has these properties:

| Name                  | Description                                                                                                         | Type                                              | Inherited from          | `JSON-LD @id`                                                    | Aliases                                                                                                            |
| --------------------- | ------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------- | ----------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                                                                                       | [`String`](./string.md)                           | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)                             | -                                                                                                                  |
| `content`             | The textual content of the link.                                                                                    | [`Inline`](./inline.md)*                          | -                       | `stencila:content`                                               | -                                                                                                                  |
| `target`              | The target of the link.                                                                                             | [`String`](./string.md)                           | -                       | [`schema:target`](https://schema.org/target)                     | -                                                                                                                  |
| `title`               | A title for the link.                                                                                               | [`String`](./string.md)                           | -                       | [`schema:headline`](https://schema.org/headline)                 | -                                                                                                                  |
| `rel`                 | The relation between the target and the current thing.                                                              | [`String`](./string.md)                           | -                       | [`schema:linkRelationship`](https://schema.org/linkRelationship) | -                                                                                                                  |
| `labelOnly`           | Only show the label of the internal target (e.g. "2"), rather than both the label type and label (e.g. "Figure 2"). | [`Boolean`](./boolean.md)                         | -                       | `stencila:labelOnly`                                             | `label-only`, `label_only`                                                                                         |
| `compilationMessages` | Messages generated while compiling the link (e.g. missing internal link or invalid external link).                  | [`CompilationMessage`](./compilation-message.md)* | -                       | `stencila:compilationMessages`                                   | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |

# Related

The `Link` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `Link` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                                    | Notes |
| ------------------------------------------------ | ------------ | ------------ | ---------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                            |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              | Encoded as [`<a>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a)                            |
| [JATS](../formats/jats.md)                       | 🔷 Low loss   | 🔷 Low loss   | Encoded as [`<ext-link>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/ext-link.html) |
| [Markdown](../formats/md.md)                     | 🔷 Low loss   | 🔷 Low loss   | Encoded using implemented function                                                                         |
| [Stencila Markdown](../formats/smd.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                            |
| [Quarto Markdown](../formats/qmd.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                            |
| [MyST Markdown](../formats/myst.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                            |
| [LLM Markdown](../formats/llmd.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                            |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                            |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                            |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                            |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                                            |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                            |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                            |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                            |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                            |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                            |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                            |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                            |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                            |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                            |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                            |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                            |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                            |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                            |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                            |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                                            |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                                            |
| [CSV](../formats/csv.md)                         |              |              |                                                                                                            |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                                            |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                                            |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                                            |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                                            |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                                            |
| [Directory](../formats/directory.md)             |              |              |                                                                                                            |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                                            |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                                            |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                                            |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                                            |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                                            |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                                            |

# Bindings

The `Link` type is represented in:

- [JSON-LD](https://stencila.org/Link.jsonld)
- [JSON Schema](https://stencila.org/Link.schema.json)
- Python class [`Link`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/link.py)
- Rust struct [`Link`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/link.rs)
- TypeScript class [`Link`](https://github.com/stencila/stencila/blob/main/ts/src/types/Link.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Link` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                | Strategy                       |
| --------- | ---------- | ---------------------------------------------------------- | ------------------------------ |
| `content` | Min+       | Generate a single fixed text value.                        | `vec![t("text")]`              |
|           | Low+       | Generate a single arbitrary, non-recursive, inline node    | `vec_inlines_non_recursive(1)` |
|           | High+      | Generate up to two arbitrary, non-recursive, inline nodes  | `vec_inlines_non_recursive(2)` |
|           | Max        | Generate up to four arbitrary, non-recursive, inline nodes | `vec_inlines_non_recursive(4)` |

# Source

This documentation was generated from [`Link.yaml`](https://github.com/stencila/stencila/blob/main/schema/Link.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
