---
title: Date
description: A calendar date encoded as a ISO 8601 string.
---

# Properties

The `Date` type has these properties:

| Name    | Description                     | Type                    | Inherited from          | `JSON-LD @id`                              | Aliases |
| ------- | ------------------------------- | ----------------------- | ----------------------- | ------------------------------------------ | ------- |
| `id`    | The identifier for this item.   | [`String`](./string.md) | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)       | -       |
| `value` | The date as an ISO 8601 string. | [`String`](./string.md) | -                       | [`schema:value`](https://schema.org/value) | -       |

# Related

The `Date` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Formats

The `Date` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                                                   | Notes |
| ------------------------------------------------ | ------------ | ------------ | ------------------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                                           |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |                                                                                                                           |
| [JATS](../formats/jats.md)                       | 🟢 No loss    | 🟢 No loss    | Encoded as [`<date>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/date.html) using special function |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              |                                                                                                                           |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |                                                                                                                           |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |                                                                                                                           |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |                                                                                                                           |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |                                                                                                                           |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                           |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                           |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                                           |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                                                           |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                           |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                           |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                           |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                           |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                           |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                                           |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                                           |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                           |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                           |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                                           |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                                           |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                           |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                           |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                                           |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                                                           |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                                                           |
| [CSV](../formats/csv.md)                         |              |              |                                                                                                                           |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                                                           |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                                                           |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                                                           |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                                                           |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                                                           |
| [Directory](../formats/directory.md)             |              |              |                                                                                                                           |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                                                           |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                                                           |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                                                           |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                                                           |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                                                           |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                                                           |

# Bindings

The `Date` type is represented in:

- [JSON-LD](https://stencila.org/Date.jsonld)
- [JSON Schema](https://stencila.org/Date.schema.json)
- Python class [`Date`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/date.py)
- Rust struct [`Date`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/date.rs)
- TypeScript class [`Date`](https://github.com/stencila/stencila/blob/main/ts/src/types/Date.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Date` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property | Complexity | Description                                                             | Strategy                              |
| -------- | ---------- | ----------------------------------------------------------------------- | ------------------------------------- |
| `value`  | Min+       | Generate a fixed date string.                                           | `String::from("2022-02-22")`          |
|          | Low+       | Generate a random date string.                                          | Regex `[0-9]{4}-[01][0-9]-[0-3][1-9]` |
|          | High+      | Generate a random string of up to 10 alphanumeric characters & hyphens. | Regex `[a-zA-Z0-9\-]{1,10}`           |
|          | Max        | Generate an arbitrary string.                                           | `String::arbitrary()`                 |

# Source

This documentation was generated from [`Date.yaml`](https://github.com/stencila/stencila/blob/main/schema/Date.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
