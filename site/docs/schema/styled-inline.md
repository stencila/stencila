---
title: Styled Inline
description: Styled inline content.
---

This type is marked as unstable and is subject to change.

# Properties

The `StyledInline` type has these properties:

| Name                  | Description                                                            | Type                                              | Inherited from          | `JSON-LD @id`                                | Aliases                                                                                                            |
| --------------------- | ---------------------------------------------------------------------- | ------------------------------------------------- | ----------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                                          | [`String`](./string.md)                           | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)         | -                                                                                                                  |
| `code`                | The code of the equation in the `styleLanguage`.                       | [`Cord`](./cord.md)                               | [`Styled`](./styled.md) | `stencila:code`                              | -                                                                                                                  |
| `styleLanguage`       | The language used for the style specification e.g. css, tw             | [`String`](./string.md)                           | [`Styled`](./styled.md) | `stencila:styleLanguage`                     | `style-language`, `style_language`                                                                                 |
| `authors`             | The authors of the code and content in the styled node.                | [`Author`](./author.md)*                          | [`Styled`](./styled.md) | [`schema:author`](https://schema.org/author) | `author`                                                                                                           |
| `provenance`          | A summary of the provenance of the code and content in the styed node. | [`ProvenanceCount`](./provenance-count.md)*       | [`Styled`](./styled.md) | `stencila:provenance`                        | -                                                                                                                  |
| `compilationDigest`   | A digest of the `code` and `styleLanguage`.                            | [`CompilationDigest`](./compilation-digest.md)    | [`Styled`](./styled.md) | `stencila:compilationDigest`                 | `compilation-digest`, `compilation_digest`                                                                         |
| `compilationMessages` | Messages generated while parsing and transpiling the style.            | [`CompilationMessage`](./compilation-message.md)* | [`Styled`](./styled.md) | `stencila:compilationMessages`               | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |
| `css`                 | A Cascading Style Sheet (CSS) transpiled from the `code` property.     | [`String`](./string.md)                           | [`Styled`](./styled.md) | `stencila:css`                               | -                                                                                                                  |
| `classList`           | A space separated list of class names associated with the node.        | [`String`](./string.md)                           | [`Styled`](./styled.md) | `stencila:classList`                         | `class-list`, `class_list`                                                                                         |
| `content`             | The content within the span.                                           | [`Inline`](./inline.md)*                          | -                       | `stencila:content`                           | -                                                                                                                  |

# Related

The `StyledInline` type is related to these types:

- Parents: [`Styled`](./styled.md)
- Children: none

# Formats

The `StyledInline` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                                                | Notes |
| ------------------------------------------------ | ------------ | ------------ | ---------------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                                        |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              | Encoded as [`<span>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span)                                  |
| [JATS](../formats/jats.md)                       | 🟢 No loss    | 🟢 No loss    | Encoded as [`<styled-content>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/styled-content.html) |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              | Encoded using implemented function                                                                                     |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |                                                                                                                        |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |                                                                                                                        |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |                                                                                                                        |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |                                                                                                                        |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                        |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                        |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                                        |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                                                        |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                        |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                        |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                        |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                        |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                        |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                        |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                                        |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                        |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                        |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                        |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                        |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                        |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                        |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                        |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                                                        |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                                                        |
| [CSV](../formats/csv.md)                         |              |              |                                                                                                                        |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                                                        |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                                                        |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                                                        |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                                                        |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                                                        |
| [Directory](../formats/directory.md)             |              |              |                                                                                                                        |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                                                        |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                                                        |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                                                        |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                                                        |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                                                        |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                                                        |

# Bindings

The `StyledInline` type is represented in:

- [JSON-LD](https://stencila.org/StyledInline.jsonld)
- [JSON Schema](https://stencila.org/StyledInline.schema.json)
- Python class [`StyledInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/styled_inline.py)
- Rust struct [`StyledInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/styled_inline.rs)
- TypeScript class [`StyledInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/StyledInline.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `StyledInline` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property        | Complexity | Description                                                                                                                       | Strategy                                                                                                                                                                    |
| --------------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `code`          | Min+       | Generate a simple fixed string of code.                                                                                           | `Cord::from("code")`                                                                                                                                                        |
|                 | Low+       | Generate a random string of up to 10 alphanumeric & space characters (trimmed). Avoid keywords used to identify other node types. | `r"[a-zA-Z0-9 ]{1,10}".prop_filter("No keywords", \|code\| !["include", "call", "if", "ifblock", "for"].contains(&code.trim())).prop_map(\|code\| Cord::from(code.trim()))` |
|                 | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                  | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`                                                                                                                                   |
|                 | Max        | Generate an arbitrary string.                                                                                                     | `String::arbitrary().prop_map(Cord::from)`                                                                                                                                  |
| `styleLanguage` | Min+       | Do not generate a style language.                                                                                                 | `None`                                                                                                                                                                      |
|                 | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                     | `option::of(r"[a-zA-Z0-9]{1,10}")`                                                                                                                                          |
|                 | Max        | Generate an arbitrary string.                                                                                                     | `option::of(String::arbitrary())`                                                                                                                                           |
| `content`       | Min+       | Generate a single fixed text value.                                                                                               | `vec![t("text")]`                                                                                                                                                           |
|                 | High+      | Generate up to two arbitrary, non-recursive, inline nodes                                                                         | `vec_inlines_non_recursive(2)`                                                                                                                                              |
|                 | Max        | Generate up to four arbitrary, non-recursive, inline nodes                                                                        | `vec_inlines_non_recursive(4)`                                                                                                                                              |

# Source

This documentation was generated from [`StyledInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/StyledInline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
