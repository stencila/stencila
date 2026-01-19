---
title: Raw Block
description: Document content in a specific format
---

The content of the block is not decoded by any codecs and is output when the encoding format
matches that of the raw block and the `render` option is used.
Analogous to node types in [Pandoc](https://github.com/jgm/pandoc-types/blob/1cf21a602535b6b263fef9548521353912115d87/src/Text/Pandoc/Definition.hs#L284) and [MultiMarkdown](https://fletcher.github.io/MultiMarkdown-6/syntax/raw.html).


# Properties

The `RawBlock` type has these properties:

| Name                  | Description                                                                             | Type                                              | Inherited from          | `JSON-LD @id`                                | Aliases                                                                                                            |
| --------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------- | ----------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                                                           | [`String`](./string.md)                           | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)         | -                                                                                                                  |
| `format`              | The format of the raw content.                                                          | [`String`](./string.md)                           | -                       | `stencila:format`                            | -                                                                                                                  |
| `content`             | The raw content.                                                                        | [`Cord`](./cord.md)                               | -                       | `stencila:content`                           | -                                                                                                                  |
| `compilationDigest`   | A digest of the `format` and `content` properties.                                      | [`CompilationDigest`](./compilation-digest.md)    | -                       | `stencila:compilationDigest`                 | `compilation-digest`, `compilation_digest`                                                                         |
| `compilationMessages` | Messages generated while parsing and transpiling the `content` into the `css` property. | [`CompilationMessage`](./compilation-message.md)* | -                       | `stencila:compilationMessages`               | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |
| `css`                 | A Cascading Style Sheet (CSS) generated from the `content`.                             | [`String`](./string.md)                           | -                       | `stencila:css`                               | -                                                                                                                  |
| `authors`             | The authors of the content.                                                             | [`Author`](./author.md)*                          | -                       | [`schema:author`](https://schema.org/author) | `author`                                                                                                           |
| `provenance`          | A summary of the provenance of the content.                                             | [`ProvenanceCount`](./provenance-count.md)*       | -                       | `stencila:provenance`                        | -                                                                                                                  |

# Related

The `RawBlock` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `RawBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                            | Notes |
| ------------------------------------------------ | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                    |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |                                    |
| [JATS](../formats/jats.md)                       |              |              |                                    |
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

The `RawBlock` type is represented in:

- [JSON-LD](https://stencila.org/RawBlock.jsonld)
- [JSON Schema](https://stencila.org/RawBlock.schema.json)
- Python class [`RawBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/raw_block.py)
- Rust struct [`RawBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/raw_block.rs)
- TypeScript class [`RawBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/RawBlock.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `RawBlock` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                                                                                                          | Strategy                                    |
| --------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| `format`  | Min+       | Fixed as Markdown                                                                                                                                    | `String::from("markdown")`                  |
|           | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                                        | `r"[a-zA-Z0-9]{1,10}"`                      |
|           | Max        | Generate an arbitrary string.                                                                                                                        | `String::arbitrary()`                       |
| `content` | Min+       | Generate a simple fixed string.                                                                                                                      | `Cord::from("content")`                     |
|           | Low+       | Generate a random string of up to 10 alphanumeric characters (exclude whitespace which <br><br>when leading or trailing causes issues for Markdown). | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)` |
|           | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                                     | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`   |
|           | Max        | Generate an arbitrary string.                                                                                                                        | `String::arbitrary().prop_map(Cord::from)`  |

# Source

This documentation was generated from [`RawBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/RawBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
