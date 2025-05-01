---
title: Code Inline
description: Inline code.
config:
  publish:
    ghost:
      type: post
      slug: code-inline
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Code
---

# Properties

The `CodeInline` type has these properties:

| Name                  | Description                              | Type                                                                                   | Inherited from                                                              | `JSON-LD @id`                                                          | Aliases                                        |
| --------------------- | ---------------------------------------- | -------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- | ---------------------------------------------------------------------- | ---------------------------------------------- |
| `id`                  | The identifier for this item.            | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                     | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)          | [`schema:id`](https://schema.org/id)                                   | -                                              |
| `code`                | The code.                                | [`Cord`](https://stencila.ghost.io/docs/reference/schema/cord)                         | [`CodeStatic`](https://stencila.ghost.io/docs/reference/schema/code-static) | `stencila:code`                                                        | -                                              |
| `programmingLanguage` | The programming language of the code.    | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                     | [`CodeStatic`](https://stencila.ghost.io/docs/reference/schema/code-static) | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | `programming-language`, `programming_language` |
| `authors`             | The authors of the code.                 | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                    | [`CodeStatic`](https://stencila.ghost.io/docs/reference/schema/code-static) | [`schema:author`](https://schema.org/author)                           | `author`                                       |
| `provenance`          | A summary of the provenance of the code. | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)* | [`CodeStatic`](https://stencila.ghost.io/docs/reference/schema/code-static) | `stencila:provenance`                                                  | -                                              |

# Related

The `CodeInline` type is related to these types:

- Parents: [`CodeStatic`](https://stencila.ghost.io/docs/reference/schema/code-static)
- Children: none

# Formats

The `CodeInline` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                               | Encoding     | Decoding   | Support                                                                                            | Notes |
| ------------------------------------------------------------------------------------ | ------------ | ---------- | -------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)                | 🟢 No loss    |            |                                                                                                    |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                        | 🟢 No loss    |            | Encoded as [`<code>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/code)              |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                        | 🟢 No loss    | 🟢 No loss  | Encoded as [`<code>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/code.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                      | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function                                                                 |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)            | 🟢 No loss    | 🟢 No loss  |                                                                                                    |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)              | 🟢 No loss    | 🟢 No loss  |                                                                                                    |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)               | 🟢 No loss    | 🟢 No loss  |                                                                                                    |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)                | 🟢 No loss    | 🟢 No loss  |                                                                                                    |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                      | 🔷 Low loss   | 🔷 Low loss |                                                                                                    |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                          | 🔷 Low loss   |            |                                                                                                    |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                  | ⚠️ High loss |            |                                                                                                    |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                      | 🔷 Low loss   | 🔷 Low loss |                                                                                                    |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)         | 🔷 Low loss   | 🔷 Low loss |                                                                                                    |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)             | 🔷 Low loss   | 🔷 Low loss |                                                                                                    |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                          | 🔷 Low loss   | 🔷 Low loss |                                                                                                    |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                        | 🟢 No loss    | 🟢 No loss  |                                                                                                    |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)                | 🟢 No loss    | 🟢 No loss  |                                                                                                    |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                      | 🟢 No loss    | 🟢 No loss  |                                                                                                    |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                   | 🟢 No loss    | 🟢 No loss  |                                                                                                    |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                        | 🟢 No loss    | 🟢 No loss  |                                                                                                    |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)              | 🟢 No loss    | 🟢 No loss  |                                                                                                    |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                        | 🟢 No loss    | 🟢 No loss  |                                                                                                    |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)             | 🔷 Low loss   | 🔷 Low loss |                                                                                                    |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)               | 🔷 Low loss   | 🔷 Low loss |                                                                                                    |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)                | 🔷 Low loss   | 🔷 Low loss |                                                                                                    |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)              |              |            |                                                                                                    |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)          |              |            |                                                                                                    |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoap) |              | 🔷 Low loss |                                                                                                    |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                      | 🔷 Low loss   |            |                                                                                                    |

# Bindings

The `CodeInline` type is represented in:

- [JSON-LD](https://stencila.org/CodeInline.jsonld)
- [JSON Schema](https://stencila.org/CodeInline.schema.json)
- Python class [`CodeInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/code_inline.py)
- Rust struct [`CodeInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_inline.rs)
- TypeScript class [`CodeInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeInline.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `CodeInline` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

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

This documentation was generated from [`CodeInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeInline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
