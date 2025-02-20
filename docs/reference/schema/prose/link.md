---
title: Link
description: A hyperlink to other pages, sections within the same document, resources, or any URL.
config:
  publish:
    ghost:
      type: page
      slug: link
      state: publish
      tags:
      - '#schema'
      - '#doc'
      - Prose
---

## Properties

The `Link` type has these properties:

| Name      | Description                                            | Type                                                                | Inherited from                                                     | `JSON-LD @id`                                                    | Aliases |
| --------- | ------------------------------------------------------ | ------------------------------------------------------------------- | ------------------------------------------------------------------ | ---------------------------------------------------------------- | ------- |
| `id`      | The identifier for this item.                          | [`String`](https://stencila.ghost.io/docs/reference/schema/string)  | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                             | -       |
| `content` | The textual content of the link.                       | [`Inline`](https://stencila.ghost.io/docs/reference/schema/inline)* | -                                                                  | `stencila:content`                                               | -       |
| `target`  | The target of the link.                                | [`String`](https://stencila.ghost.io/docs/reference/schema/string)  | -                                                                  | [`schema:target`](https://schema.org/target)                     | -       |
| `title`   | A title for the link.                                  | [`String`](https://stencila.ghost.io/docs/reference/schema/string)  | -                                                                  | [`schema:headline`](https://schema.org/headline)                 | -       |
| `rel`     | The relation between the target and the current thing. | [`String`](https://stencila.ghost.io/docs/reference/schema/string)  | -                                                                  | [`schema:linkRelationship`](https://schema.org/linkRelationship) | -       |

## Related

The `Link` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: none

## Formats

The `Link` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                       | Encoding     | Decoding   | Support                                                                                                    | Notes |
| ---------------------------------------------------------------------------- | ------------ | ---------- | ---------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)        | 🟢 No loss    |            |                                                                                                            |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                | 🔷 Low loss   |            | Encoded as [`<a>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a)                            |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                | 🔷 Low loss   | 🔷 Low loss | Encoded as [`<ext-link>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/ext-link.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)              | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function                                                                         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)    | 🔷 Low loss   | 🔷 Low loss |                                                                                                            |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)      | 🔷 Low loss   | 🔷 Low loss |                                                                                                            |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)       | 🔷 Low loss   | 🔷 Low loss |                                                                                                            |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)        | 🔷 Low loss   | 🔷 Low loss |                                                                                                            |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)              | 🔷 Low loss   | 🔷 Low loss |                                                                                                            |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                  | 🔷 Low loss   |            |                                                                                                            |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)          | ⚠️ High loss |            |                                                                                                            |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)              | 🔷 Low loss   | 🔷 Low loss |                                                                                                            |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx) | 🔷 Low loss   | 🔷 Low loss |                                                                                                            |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)     | 🔷 Low loss   | 🔷 Low loss |                                                                                                            |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                  | 🔷 Low loss   | 🔷 Low loss |                                                                                                            |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                | 🟢 No loss    | 🟢 No loss  |                                                                                                            |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)        | 🟢 No loss    | 🟢 No loss  |                                                                                                            |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)              | 🟢 No loss    | 🟢 No loss  |                                                                                                            |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)           | 🟢 No loss    | 🟢 No loss  |                                                                                                            |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                | 🟢 No loss    | 🟢 No loss  |                                                                                                            |
| [CBOR+Zstandard](https://stencila.ghost.io/docs/reference/formats/cbor.zstd) | 🟢 No loss    | 🟢 No loss  |                                                                                                            |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                | 🟢 No loss    | 🟢 No loss  |                                                                                                            |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)     | 🔷 Low loss   | 🔷 Low loss |                                                                                                            |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)       | 🔷 Low loss   | 🔷 Low loss |                                                                                                            |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)        | 🔷 Low loss   | 🔷 Low loss |                                                                                                            |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)      |              |            |                                                                                                            |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)  |              |            |                                                                                                            |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)              | 🔷 Low loss   |            |                                                                                                            |

## Bindings

The `Link` type is represented in:

- [JSON-LD](https://stencila.org/Link.jsonld)
- [JSON Schema](https://stencila.org/Link.schema.json)
- Python class [`Link`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/link.py)
- Rust struct [`Link`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/link.rs)
- TypeScript class [`Link`](https://github.com/stencila/stencila/blob/main/ts/src/types/Link.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `Link` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                | Strategy                       |
| --------- | ---------- | ---------------------------------------------------------- | ------------------------------ |
| `content` | Min+       | Generate a single fixed text value.                        | `vec![t("text")]`              |
|           | Low+       | Generate a single arbitrary, non-recursive, inline node    | `vec_inlines_non_recursive(1)` |
|           | High+      | Generate up to two arbitrary, non-recursive, inline nodes  | `vec_inlines_non_recursive(2)` |
|           | Max        | Generate up to four arbitrary, non-recursive, inline nodes | `vec_inlines_non_recursive(4)` |

## Source

This documentation was generated from [`Link.yaml`](https://github.com/stencila/stencila/blob/main/schema/Link.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
