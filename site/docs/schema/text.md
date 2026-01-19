---
title: Text
description: Textual content.
---

Intended mostly for use for inline text e.g. the text in a paragraph.

Differs from the primitive `String` type in that it has a `type` and `id` property.
The `id` property allows use to identify text nodes with a sequence of inline nodes
for better diffing.

Also, in Rust, the `value` property is implemented as a CRDT.


# Properties

The `Text` type has these properties:

| Name                  | Description                                  | Type                                              | Inherited from          | `JSON-LD @id`                              | Aliases                                                                                                            |
| --------------------- | -------------------------------------------- | ------------------------------------------------- | ----------------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                | [`String`](./string.md)                           | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)       | -                                                                                                                  |
| `value`               | The value of the text content                | [`Cord`](./cord.md)                               | -                       | [`schema:value`](https://schema.org/value) | -                                                                                                                  |
| `compilationMessages` | Messages generated while compiling the text. | [`CompilationMessage`](./compilation-message.md)* | -                       | `stencila:compilationMessages`             | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |

# Related

The `Text` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `Text` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                               | Notes |
| ------------------------------------------------ | ------------ | ------------ | ------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                       |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              | Encoded as [`<span>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span) |
| [JATS](../formats/jats.md)                       | 🟢 No loss    | 🟢 No loss    | Encoded using special function                                                        |
| [Markdown](../formats/md.md)                     | 🟢 No loss    | 🟢 No loss    | Encoded using implemented function                                                    |
| [Stencila Markdown](../formats/smd.md)           | 🟢 No loss    | 🟢 No loss    |                                                                                       |
| [Quarto Markdown](../formats/qmd.md)             | 🟢 No loss    | 🟢 No loss    |                                                                                       |
| [MyST Markdown](../formats/myst.md)              | 🟢 No loss    | 🟢 No loss    |                                                                                       |
| [LLM Markdown](../formats/llmd.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                       |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                       |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                       |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                       |
| [Plain text](../formats/text.md)                 | 🟢 No loss    |              |                                                                                       |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                       |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                       |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                       |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                       |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                       |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                       |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                       |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                       |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                       |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                       |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                       |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                       |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                       |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                       |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                       |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                       |
| [CSV](../formats/csv.md)                         |              |              |                                                                                       |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                       |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                       |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                       |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                       |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                       |
| [Directory](../formats/directory.md)             |              |              |                                                                                       |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                       |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                       |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                       |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                       |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                       |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                       |

# Bindings

The `Text` type is represented in:

- [JSON-LD](https://stencila.org/Text.jsonld)
- [JSON Schema](https://stencila.org/Text.schema.json)
- Python class [`Text`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/text.py)
- Rust struct [`Text`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/text.rs)
- TypeScript class [`Text`](https://github.com/stencila/stencila/blob/main/ts/src/types/Text.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Text` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property | Complexity | Description                                                                                                                    | Strategy                                                        |
| -------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------- |
| `value`  | Min+       | Generate a fixed string of text.                                                                                               | `Cord::from("text")`                                            |
|          | Low+       | Generate a random string of up to 10 alphanumeric characters.                                                                  | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)`                     |
|          | High+      | Generate a random string of up to 100 alphanumeric characters, some special characters commonly used in prose, and whitespace. | `r"[a-zA-Z0-9 \t\-_.!?*+-/()'<>=]{1,100}".prop_map(Cord::from)` |
|          | Max        | Generate an arbitrary string.                                                                                                  | `String::arbitrary().prop_map(Cord::from)`                      |

# Source

This documentation was generated from [`Text.yaml`](https://github.com/stencila/stencila/blob/main/schema/Text.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
