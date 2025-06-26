---
title: List Item
description: A single item in a list.
config:
  publish:
    ghost:
      type: post
      slug: list-item
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

This is an implementation, and extension, of schema.org [`ListItem`](https://schema.org/ListItem).
It extends schema.ord `ListItem` by adding `content` and `isChecked` properties.

Analogues of `ListItem` in other schema include:
  - JATS XML `<list-item>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.2/element/list-item.html)
  - HTML [`<li>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li)
  - MDAST [`ListItem`](https://github.com/syntax-tree/mdast#listitem)
  - OpenDocument [`<text:list-item>`](http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part1.html#__RefHeading__1415154_253892949)


# Properties

The `ListItem` type has these properties:

| Name             | Description                                                | Type                                                                                                                                                       | Inherited from                                                     | `JSON-LD @id`                                              | Aliases                                                                                   |
| ---------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ---------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `id`             | The identifier for this item.                              | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                       | -                                                                                         |
| `alternateNames` | Alternate names (aliases) for the item.                    | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:alternateName`](https://schema.org/alternateName) | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` |
| `description`    | A description of the item.                                 | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:description`](https://schema.org/description)     | -                                                                                         |
| `identifiers`    | Any kind of identifier for any kind of Thing.              | ([`PropertyValue`](https://stencila.ghost.io/docs/reference/schema/property-value) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string))* | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:identifier`](https://schema.org/identifier)       | `identifier`                                                                              |
| `images`         | Images of the item.                                        | [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)*                                                                             | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:image`](https://schema.org/image)                 | `image`                                                                                   |
| `name`           | The name of the item.                                      | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:name`](https://schema.org/name)                   | -                                                                                         |
| `url`            | The URL of the item.                                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:url`](https://schema.org/url)                     | -                                                                                         |
| `content`        | The content of the list item.                              | [`Block`](https://stencila.ghost.io/docs/reference/schema/block)*                                                                                          | -                                                                  | `stencila:content`                                         | -                                                                                         |
| `item`           | The item represented by this list item.                    | [`Node`](https://stencila.ghost.io/docs/reference/schema/node)                                                                                             | -                                                                  | [`schema:item`](https://schema.org/item)                   | -                                                                                         |
| `isChecked`      | A flag to indicate if this list item is checked.           | [`Boolean`](https://stencila.ghost.io/docs/reference/schema/boolean)                                                                                       | -                                                                  | `stencila:isChecked`                                       | `is-checked`, `is_checked`                                                                |
| `position`       | The position of the item in a series or sequence of items. | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)                                                                                       | -                                                                  | [`schema:position`](https://schema.org/position)           | -                                                                                         |

# Related

The `ListItem` type is related to these types:

- Parents: [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)
- Children: none

# Formats

The `ListItem` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding   | Support                                                                                                      | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ---------- | ------------------------------------------------------------------------------------------------------------ | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |            |                                                                                                              |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |            | Encoded as [`<li>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li)                            |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       | 🔷 Low loss   |            | Encoded as [`<list-item>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/list-item.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function                                                                           |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | 🔷 Low loss   | 🔷 Low loss |                                                                                                              |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | 🔷 Low loss   | 🔷 Low loss |                                                                                                              |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | 🔷 Low loss   | 🔷 Low loss |                                                                                                              |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | 🔷 Low loss   | 🔷 Low loss |                                                                                                              |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss |                                                                                                              |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss |                                                                                                              |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss |            |                                                                                                              |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |            |                                                                                                              |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss |                                                                                                              |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss |                                                                                                              |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |            |                                                                                                              |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss |                                                                                                              |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss |                                                                                                              |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss  |                                                                                                              |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss  |                                                                                                              |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss  |                                                                                                              |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss  |                                                                                                              |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss  |                                                                                                              |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss  |                                                                                                              |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss  |                                                                                                              |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss |                                                                                                              |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss |                                                                                                              |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss |                                                                                                              |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |            |                                                                                                              |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |            |                                                                                                              |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |            |                                                                                                              |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss |                                                                                                              |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss |                                                                                                              |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |            |                                                                                                              |

# Bindings

The `ListItem` type is represented in:

- [JSON-LD](https://stencila.org/ListItem.jsonld)
- [JSON Schema](https://stencila.org/ListItem.schema.json)
- Python class [`ListItem`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/list_item.py)
- Rust struct [`ListItem`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/list_item.rs)
- TypeScript class [`ListItem`](https://github.com/stencila/stencila/blob/main/ts/src/types/ListItem.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `ListItem` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                     | Strategy                  |
| --------- | ---------- | ----------------------------------------------- | ------------------------- |
| `content` | Min+       | Generate a single, arbitrary, paragraph         | `vec_paragraphs(1)`       |
|           | Low+       | Generate one, arbitrary, non-list block         | `vec_blocks_list_item(1)` |
|           | High+      | Generate up to two, arbitrary, non-list blocks  | `vec_blocks_list_item(2)` |
|           | Max        | Generate up to four, arbitrary, non-list blocks | `vec_blocks_list_item(4)` |

# Source

This documentation was generated from [`ListItem.yaml`](https://github.com/stencila/stencila/blob/main/schema/ListItem.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
