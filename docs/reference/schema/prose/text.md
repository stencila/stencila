---
title: Text
description: Textual content.
config:
  publish:
    ghost:
      type: post
      slug: text
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

Intended mostly for use for inline text e.g. the text in a paragraph.

Differs from the primitive `String` type in that it has a `type` and `id` property.
The `id` property allows use to identify text nodes with a sequence of inline nodes
for better diffing.

Also, in Rust, the `value` property is implemented as a CRDT.


# Properties

The `Text` type has these properties:

| Name    | Description                   | Type                                                               | Inherited from                                                     | `JSON-LD @id`                              | Aliases |
| ------- | ----------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------ | ------------------------------------------ | ------- |
| `id`    | The identifier for this item. | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)       | -       |
| `value` | The value of the text content | [`Cord`](https://stencila.ghost.io/docs/reference/schema/cord)     | -                                                                  | [`schema:value`](https://schema.org/value) | -       |

# Related

The `Text` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `Text` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                               | Encoding     | Decoding   | Support                                                                               | Notes |
| ------------------------------------------------------------------------------------ | ------------ | ---------- | ------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)                | 🟢 No loss    |            |                                                                                       |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                        | 🟢 No loss    |            | Encoded as [`<span>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span) |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                        | 🟢 No loss    | 🟢 No loss  | Encoded using special function                                                        |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                      | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function                                                    |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)            | 🟢 No loss    | 🟢 No loss  |                                                                                       |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)              | 🟢 No loss    | 🟢 No loss  |                                                                                       |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)               | 🟢 No loss    | 🟢 No loss  |                                                                                       |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)                | 🟢 No loss    | 🟢 No loss  |                                                                                       |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                      | 🔷 Low loss   | 🔷 Low loss |                                                                                       |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                      | 🔷 Low loss   | 🔷 Low loss |                                                                                       |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                          | ⚠️ High loss |            |                                                                                       |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                  | 🟢 No loss    |            |                                                                                       |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                      | 🔷 Low loss   | 🔷 Low loss |                                                                                       |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)         | 🔷 Low loss   | 🔷 Low loss |                                                                                       |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)           |              |            |                                                                                       |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)             | 🔷 Low loss   | 🔷 Low loss |                                                                                       |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                          | 🔷 Low loss   | 🔷 Low loss |                                                                                       |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                        | 🟢 No loss    | 🟢 No loss  |                                                                                       |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)                | 🟢 No loss    | 🟢 No loss  |                                                                                       |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                      | 🟢 No loss    | 🟢 No loss  |                                                                                       |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                   | 🟢 No loss    | 🟢 No loss  |                                                                                       |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                        | 🟢 No loss    | 🟢 No loss  |                                                                                       |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)              | 🟢 No loss    | 🟢 No loss  |                                                                                       |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                        | 🟢 No loss    | 🟢 No loss  |                                                                                       |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)             | 🔷 Low loss   | 🔷 Low loss |                                                                                       |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)               | 🔷 Low loss   | 🔷 Low loss |                                                                                       |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)                | 🔷 Low loss   | 🔷 Low loss |                                                                                       |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                          | ⚠️ High loss |            |                                                                                       |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)              |              |            |                                                                                       |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)          |              |            |                                                                                       |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoap) |              | 🔷 Low loss |                                                                                       |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                      | 🔷 Low loss   |            |                                                                                       |

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
