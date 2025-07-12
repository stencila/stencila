---
title: Raw Block
description: Document content in a specific format
config:
  publish:
    ghost:
      type: post
      slug: raw-block
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Other
---

The content of the block is not decoded by any codecs and is output when the encoding format
matches that of the raw block and the `render` option is used.
Analogous to node types in [Pandoc](https://github.com/jgm/pandoc-types/blob/1cf21a602535b6b263fef9548521353912115d87/src/Text/Pandoc/Definition.hs#L284) and [MultiMarkdown](https://fletcher.github.io/MultiMarkdown-6/syntax/raw.html).


# Properties

The `RawBlock` type has these properties:

| Name                  | Description                                                                             | Type                                                                                         | Inherited from                                                     | `JSON-LD @id`                                | Aliases                                                                                                            |
| --------------------- | --------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                                                           | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)         | -                                                                                                                  |
| `format`              | The format of the raw content.                                                          | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | -                                                                  | `stencila:format`                            | -                                                                                                                  |
| `content`             | The raw content.                                                                        | [`Cord`](https://stencila.ghost.io/docs/reference/schema/cord)                               | -                                                                  | `stencila:content`                           | -                                                                                                                  |
| `compilationDigest`   | A digest of the `format` and `content` properties.                                      | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)    | -                                                                  | `stencila:compilationDigest`                 | `compilation-digest`, `compilation_digest`                                                                         |
| `compilationMessages` | Messages generated while parsing and transpiling the `content` into the `css` property. | [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message)* | -                                                                  | `stencila:compilationMessages`               | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |
| `css`                 | A Cascading Style Sheet (CSS) generated from the `content`.                             | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | -                                                                  | `stencila:css`                               | -                                                                                                                  |
| `authors`             | The authors of the content.                                                             | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                          | -                                                                  | [`schema:author`](https://schema.org/author) | `author`                                                                                                           |
| `provenance`          | A summary of the provenance of the content.                                             | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)*       | -                                                                  | `stencila:provenance`                        | -                                                                                                                  |

# Related

The `RawBlock` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

# Formats

The `RawBlock` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding     | Support                            | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ------------ | ---------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |              |                                    |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |              |                                    |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |              |              |                                    |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | ⚠️ High loss |              | Encoded using implemented function |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | ⚠️ High loss |              |                                    |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | ⚠️ High loss |              |                                    |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | ⚠️ High loss |              |                                    |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | ⚠️ High loss |              |                                    |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss | ⚠️ High loss |                                    |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |              |                                    |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |              |                                    |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss    |                                    |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss    |                                    |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss    |                                    |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss   |                                    |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |              |                                    |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |              |                                    |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |              |                                    |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss   |                                    |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss   |                                    |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |              |                                    |

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
