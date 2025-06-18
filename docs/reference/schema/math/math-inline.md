---
title: Math Inline
description: A fragment of math, e.g a variable name, to be treated as inline content.
config:
  publish:
    ghost:
      type: post
      slug: math-inline
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Math
---

# Properties

The `MathInline` type has these properties:

| Name                  | Description                                                         | Type                                                                                         | Inherited from                                                     | `JSON-LD @id`                                | Aliases                                                                                                            |
| --------------------- | ------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `id`                  | The identifier for this item.                                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)         | -                                                                                                                  |
| `code`                | The code of the equation in the `mathLanguage`.                     | [`Cord`](https://stencila.ghost.io/docs/reference/schema/cord)                               | [`Math`](https://stencila.ghost.io/docs/reference/schema/math)     | `stencila:code`                              | -                                                                                                                  |
| `mathLanguage`        | The language used for the equation e.g tex, mathml, asciimath.      | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | [`Math`](https://stencila.ghost.io/docs/reference/schema/math)     | `stencila:mathLanguage`                      | `math-language`, `math_language`                                                                                   |
| `authors`             | The authors of the math.                                            | [`Author`](https://stencila.ghost.io/docs/reference/schema/author)*                          | [`Math`](https://stencila.ghost.io/docs/reference/schema/math)     | [`schema:author`](https://schema.org/author) | `author`                                                                                                           |
| `provenance`          | A summary of the provenance of the math.                            | [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count)*       | [`Math`](https://stencila.ghost.io/docs/reference/schema/math)     | `stencila:provenance`                        | -                                                                                                                  |
| `compilationDigest`   | A digest of the `code` and `mathLanguage`.                          | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)    | [`Math`](https://stencila.ghost.io/docs/reference/schema/math)     | `stencila:compilationDigest`                 | `compilation-digest`, `compilation_digest`                                                                         |
| `compilationMessages` | Messages generated while parsing and compiling the math expression. | [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message)* | [`Math`](https://stencila.ghost.io/docs/reference/schema/math)     | `stencila:compilationMessages`               | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message` |
| `mathml`              | The MathML transpiled from the `code`.                              | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                           | [`Math`](https://stencila.ghost.io/docs/reference/schema/math)     | `stencila:mathml`                            | -                                                                                                                  |
| `images`              | Images of the math.                                                 | [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)*               | [`Math`](https://stencila.ghost.io/docs/reference/schema/math)     | [`schema:image`](https://schema.org/image)   | `image`                                                                                                            |

# Related

The `MathInline` type is related to these types:

- Parents: [`Math`](https://stencila.ghost.io/docs/reference/schema/math)
- Children: none

# Formats

The `MathInline` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                               | Encoding     | Decoding   | Support                                                                                                                                       | Notes |
| ------------------------------------------------------------------------------------ | ------------ | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)                | 🟢 No loss    |            |                                                                                                                                               |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                        | 🔷 Low loss   |            | Encoded as [`<math>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/math)                                                         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                        | 🟢 No loss    | 🔷 Low loss | Encoded as [`<inline-formula>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/inline-formula.html) using special function |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                      | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function                                                                                                            |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)            | 🟢 No loss    | 🟢 No loss  |                                                                                                                                               |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)              | 🟢 No loss    | 🟢 No loss  |                                                                                                                                               |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)               | 🟢 No loss    | 🟢 No loss  |                                                                                                                                               |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)                | 🟢 No loss    | 🟢 No loss  |                                                                                                                                               |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                      | 🔷 Low loss   | 🔷 Low loss |                                                                                                                                               |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                      | 🔷 Low loss   | 🔷 Low loss |                                                                                                                                               |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                          | ⚠️ High loss |            |                                                                                                                                               |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                  | ⚠️ High loss |            |                                                                                                                                               |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                      | 🔷 Low loss   | 🔷 Low loss |                                                                                                                                               |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)         | 🔷 Low loss   | 🔷 Low loss |                                                                                                                                               |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)             | 🔷 Low loss   | 🔷 Low loss |                                                                                                                                               |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                          | 🔷 Low loss   | 🔷 Low loss |                                                                                                                                               |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                        | 🟢 No loss    | 🟢 No loss  |                                                                                                                                               |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)                | 🟢 No loss    | 🟢 No loss  |                                                                                                                                               |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                      | 🟢 No loss    | 🟢 No loss  |                                                                                                                                               |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                   | 🟢 No loss    | 🟢 No loss  |                                                                                                                                               |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                        | 🟢 No loss    | 🟢 No loss  |                                                                                                                                               |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)              | 🟢 No loss    | 🟢 No loss  |                                                                                                                                               |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                        | 🟢 No loss    | 🟢 No loss  |                                                                                                                                               |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)             | 🔷 Low loss   | 🔷 Low loss |                                                                                                                                               |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)               | 🔷 Low loss   | 🔷 Low loss |                                                                                                                                               |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)                | 🔷 Low loss   | 🔷 Low loss |                                                                                                                                               |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                          | ⚠️ High loss |            |                                                                                                                                               |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)              |              |            |                                                                                                                                               |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)          |              |            |                                                                                                                                               |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoap) |              | 🔷 Low loss |                                                                                                                                               |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                      | 🔷 Low loss   |            |                                                                                                                                               |

# Bindings

The `MathInline` type is represented in:

- [JSON-LD](https://stencila.org/MathInline.jsonld)
- [JSON Schema](https://stencila.org/MathInline.schema.json)
- Python class [`MathInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/math_inline.py)
- Rust struct [`MathInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/math_inline.rs)
- TypeScript class [`MathInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/MathInline.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `MathInline` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

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

This documentation was generated from [`MathInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/MathInline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
