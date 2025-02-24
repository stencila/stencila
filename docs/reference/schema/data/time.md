---
title: Time
description: A point in time recurring on multiple days.
config:
  publish:
    ghost:
      type: post
      slug: time
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Data
---

# Properties

The `Time` type has these properties:

| Name    | Description                                                       | Type                                                               | Inherited from                                                     | `JSON-LD @id`                              | Aliases |
| ------- | ----------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------ | ------------------------------------------ | ------- |
| `id`    | The identifier for this item.                                     | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)       | -       |
| `value` | The time of day as a string in format `hh:mm:ss[Z\|(+\|-)hh:mm]`. | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | -                                                                  | [`schema:value`](https://schema.org/value) | -       |

# Related

The `Time` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `Time` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                       | Encoding     | Decoding   | Support                                                                                                                   | Notes |
| ---------------------------------------------------------------------------- | ------------ | ---------- | ------------------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)        | 🟢 No loss    |            |                                                                                                                           |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                | 🔷 Low loss   |            |                                                                                                                           |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                | 🟢 No loss    | 🟢 No loss  | Encoded as [`<time>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/time.html) using special function |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)              | ⚠️ High loss |            |                                                                                                                           |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)    | ⚠️ High loss |            |                                                                                                                           |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)      | ⚠️ High loss |            |                                                                                                                           |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)       | ⚠️ High loss |            |                                                                                                                           |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)        | ⚠️ High loss |            |                                                                                                                           |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)              | 🔷 Low loss   | 🔷 Low loss |                                                                                                                           |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                  | 🔷 Low loss   |            |                                                                                                                           |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)          | ⚠️ High loss |            |                                                                                                                           |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)              | 🔷 Low loss   | 🔷 Low loss |                                                                                                                           |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx) | 🔷 Low loss   | 🔷 Low loss |                                                                                                                           |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)     | 🔷 Low loss   | 🔷 Low loss |                                                                                                                           |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                  | 🔷 Low loss   | 🔷 Low loss |                                                                                                                           |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                | 🟢 No loss    | 🟢 No loss  |                                                                                                                           |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)        | 🟢 No loss    | 🟢 No loss  |                                                                                                                           |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)              | 🟢 No loss    | 🟢 No loss  |                                                                                                                           |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)           | 🟢 No loss    | 🟢 No loss  |                                                                                                                           |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                | 🟢 No loss    | 🟢 No loss  |                                                                                                                           |
| [CBOR+Zstandard](https://stencila.ghost.io/docs/reference/formats/cbor.zstd) | 🟢 No loss    | 🟢 No loss  |                                                                                                                           |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                | 🟢 No loss    | 🟢 No loss  |                                                                                                                           |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)     | 🔷 Low loss   | 🔷 Low loss |                                                                                                                           |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)       | 🔷 Low loss   | 🔷 Low loss |                                                                                                                           |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)        | 🔷 Low loss   | 🔷 Low loss |                                                                                                                           |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)      |              |            |                                                                                                                           |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)  |              |            |                                                                                                                           |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)              | 🔷 Low loss   |            |                                                                                                                           |

# Bindings

The `Time` type is represented in:

- [JSON-LD](https://stencila.org/Time.jsonld)
- [JSON Schema](https://stencila.org/Time.schema.json)
- Python class [`Time`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/time.py)
- Rust struct [`Time`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/time.rs)
- TypeScript class [`Time`](https://github.com/stencila/stencila/blob/main/ts/src/types/Time.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Time` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property | Complexity | Description                                                                     | Strategy                                                                       |
| -------- | ---------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `value`  | Min+       | Generate a fixed date-time string.                                              | `String::from("2022-02-22T22:22:22")`                                          |
|          | Low+       | Generate a random date-time string.                                             | Regex `[0-2][0-9]:[0-5][0-9]:[0-5][0-9]\.[0-9]+([+-][0-2][0-9]:[0-5][0-9]\|Z)` |
|          | High+      | Generate a random string of up to 20 alphanumeric characters, colons & hyphens. | Regex `[a-zA-Z0-9\-:]{1,20}`                                                   |
|          | Max        | Generate an arbitrary string.                                                   | `String::arbitrary()`                                                          |

# Source

This documentation was generated from [`Time.yaml`](https://github.com/stencila/stencila/blob/main/schema/Time.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
