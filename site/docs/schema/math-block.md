---
title: Math Block
description: A block of math, e.g an equation, to be treated as block content.
---

# Properties

The `MathBlock` type has these properties:

| Name                  | Description                                                         | Type                                              | Inherited from          | `JSON-LD @id`                                | Aliases                                                                                                            |
| --------------------- | ------------------------------------------------------------------- | ------------------------------------------------- | ----------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                                       | [`String`](./string.md)                           | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)         | -                                                                                                                  |
| `code`                | The code of the equation in the `mathLanguage`.                     | [`Cord`](./cord.md)                               | [`Math`](./math.md)     | `stencila:code`                              | -                                                                                                                  |
| `mathLanguage`        | The language used for the equation e.g tex, mathml, asciimath.      | [`String`](./string.md)                           | [`Math`](./math.md)     | `stencila:mathLanguage`                      | `math-language`, `math_language`                                                                                   |
| `authors`             | The authors of the math.                                            | [`Author`](./author.md)*                          | [`Math`](./math.md)     | [`schema:author`](https://schema.org/author) | `author`                                                                                                           |
| `provenance`          | A summary of the provenance of the math.                            | [`ProvenanceCount`](./provenance-count.md)*       | [`Math`](./math.md)     | `stencila:provenance`                        | -                                                                                                                  |
| `compilationDigest`   | A digest of the `code` and `mathLanguage`.                          | [`CompilationDigest`](./compilation-digest.md)    | [`Math`](./math.md)     | `stencila:compilationDigest`                 | `compilation-digest`, `compilation_digest`                                                                         |
| `compilationMessages` | Messages generated while parsing and compiling the math expression. | [`CompilationMessage`](./compilation-message.md)* | [`Math`](./math.md)     | `stencila:compilationMessages`               | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |
| `mathml`              | The MathML transpiled from the `code`.                              | [`String`](./string.md)                           | [`Math`](./math.md)     | `stencila:mathml`                            | -                                                                                                                  |
| `images`              | Images of the math.                                                 | [`ImageObject`](./image-object.md)*               | [`Math`](./math.md)     | [`schema:image`](https://schema.org/image)   | `image`                                                                                                            |
| `label`               | A short label for the math block.                                   | [`String`](./string.md)                           | -                       | `stencila:label`                             | -                                                                                                                  |
| `labelAutomatically`  | Whether the label should be automatically updated.                  | [`Boolean`](./boolean.md)                         | -                       | `stencila:labelAutomatically`                | `label-automatically`, `label_automatically`                                                                       |

# Related

The `MathBlock` type is related to these types:

- Parents: [`Math`](./math.md)
- Children: none

# Formats

The `MathBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                                                                   | Notes |
| ------------------------------------------------ | ------------ | ------------ | ----------------------------------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                                                           |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              | Encoded as [`<math>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/math)                                                     |
| [JATS](../formats/jats.md)                       | 🟢 No loss    | 🔷 Low loss   | Encoded as [`<disp-formula>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/disp-formula.html) using special function |
| [Markdown](../formats/md.md)                     | 🟢 No loss    | 🟢 No loss    | Encoded using implemented function                                                                                                        |
| [Stencila Markdown](../formats/smd.md)           | 🟢 No loss    | 🟢 No loss    |                                                                                                                                           |
| [Quarto Markdown](../formats/qmd.md)             | 🟢 No loss    | 🟢 No loss    |                                                                                                                                           |
| [MyST Markdown](../formats/myst.md)              | 🟢 No loss    | 🟢 No loss    |                                                                                                                                           |
| [LLM Markdown](../formats/llmd.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                                           |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                           |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                           |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                                                           |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                                                                           |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                           |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                           |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                           |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                           |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                                           |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                                           |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                                                           |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                                           |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                                           |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                                           |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                                           |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                           |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                           |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                                           |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                                                                           |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                                                                           |
| [CSV](../formats/csv.md)                         |              |              |                                                                                                                                           |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                                                                           |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                                                                           |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                                                                           |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                                                                           |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                                                                           |
| [Directory](../formats/directory.md)             |              |              |                                                                                                                                           |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                                                                           |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                                                                           |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                                                                           |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                                                                           |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                                                                           |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                                                                           |

# Bindings

The `MathBlock` type is represented in:

- [JSON-LD](https://stencila.org/MathBlock.jsonld)
- [JSON Schema](https://stencila.org/MathBlock.schema.json)
- Python class [`MathBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/math_block.py)
- Rust struct [`MathBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/math_block.rs)
- TypeScript class [`MathBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/MathBlock.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `MathBlock` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property       | Complexity | Description                                                                                                                                          | Strategy                                    |
| -------------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| `code`         | Min+       | Generate a simple fixed string of math.                                                                                                              | `Cord::from("math")`                        |
|                | Low+       | Generate a random string of up to 10 alphanumeric characters (exclude whitespace which <br><br>when leading or trailing causes issues for Markdown). | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)` |
|                | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                                     | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`   |
|                | Max        | Generate an arbitrary string.                                                                                                                        | `String::arbitrary().prop_map(Cord::from)`  |
| `mathLanguage` | Min+       | Fixed as TeX (for testing with Markdown which uses dollars to delimit TeX by default)                                                                | `Some(String::from("tex"))`                 |
|                | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                                        | `option::of(r"[a-zA-Z0-9]{1,10}")`          |
|                | Max        | Generate an arbitrary string.                                                                                                                        | `option::of(String::arbitrary())`           |

# Source

This documentation was generated from [`MathBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/MathBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
