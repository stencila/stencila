---
title: Code Block
description: A code block.
---

# Properties

The `CodeBlock` type has these properties:

| Name                  | Description                              | Type                                        | Inherited from                   | `JSON-LD @id`                                                          | Aliases                                        |
| --------------------- | ---------------------------------------- | ------------------------------------------- | -------------------------------- | ---------------------------------------------------------------------- | ---------------------------------------------- |
| `id`                  | The identifier for this item.            | [`String`](./string.md)                     | [`Entity`](./entity.md)          | [`schema:id`](https://schema.org/id)                                   | -                                              |
| `code`                | The code.                                | [`Cord`](./cord.md)                         | [`CodeStatic`](./code-static.md) | `stencila:code`                                                        | -                                              |
| `programmingLanguage` | The programming language of the code.    | [`String`](./string.md)                     | [`CodeStatic`](./code-static.md) | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | `programming-language`, `programming_language` |
| `authors`             | The authors of the code.                 | [`Author`](./author.md)*                    | [`CodeStatic`](./code-static.md) | [`schema:author`](https://schema.org/author)                           | `author`                                       |
| `provenance`          | A summary of the provenance of the code. | [`ProvenanceCount`](./provenance-count.md)* | [`CodeStatic`](./code-static.md) | `stencila:provenance`                                                  | -                                              |

# Related

The `CodeBlock` type is related to these types:

- Parents: [`CodeStatic`](./code-static.md)
- Children: none

# Formats

The `CodeBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                            | Notes |
| ------------------------------------------------ | ------------ | ------------ | -------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                    |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              | Encoded as [`<pre>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/pre)                |
| [JATS](../formats/jats.md)                       | 🟢 No loss    |              | Encoded as [`<code>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/code.html) |
| [Markdown](../formats/md.md)                     | 🟢 No loss    | 🟢 No loss    | Encoded using implemented function                                                                 |
| [Stencila Markdown](../formats/smd.md)           | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [Quarto Markdown](../formats/qmd.md)             | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [MyST Markdown](../formats/myst.md)              | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [LLM Markdown](../formats/llmd.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                    |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                                    |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                    |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                    |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                                    |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                                    |
| [CSV](../formats/csv.md)                         |              |              |                                                                                                    |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                                    |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                                    |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                                    |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                                    |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                                    |
| [Directory](../formats/directory.md)             |              |              |                                                                                                    |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                                    |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                                    |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                                    |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                                    |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                                    |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                                    |

# Bindings

The `CodeBlock` type is represented in:

- [JSON-LD](https://stencila.org/CodeBlock.jsonld)
- [JSON Schema](https://stencila.org/CodeBlock.schema.json)
- Python class [`CodeBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/code_block.py)
- Rust struct [`CodeBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_block.rs)
- TypeScript class [`CodeBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeBlock.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `CodeBlock` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property              | Complexity | Description                                                                                                                    | Strategy                                      |
| --------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------- |
| `code`                | Min+       | Generate a simple fixed string of code.                                                                                        | `Cord::from("code")`                          |
|                       | Low+       | Generate a random string of up to 10 alphanumeric characters (exclude whitespace which<br><br>can be problematic in Markdown). | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)`   |
|                       | High+      | Generate a random string of up to 100 characters (excluding control characters).                                               | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`     |
|                       | Max        | Generate an arbitrary string.                                                                                                  | `String::arbitrary().prop_map(Cord::from)`    |
| `programmingLanguage` | Min+       | Do not generate a programming language.                                                                                        | `None`                                        |
|                       | Low+       | Generate one of the well known programming language short names.                                                               | `option::of(r"(cpp)\|(js)\|(py)\|(r)\|(ts)")` |
|                       | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                  | `option::of(r"[a-zA-Z0-9]{1,10}")`            |
|                       | Max        | Generate an arbitrary string.                                                                                                  | `option::of(String::arbitrary())`             |

# Source

This documentation was generated from [`CodeBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
