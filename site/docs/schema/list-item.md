---
title: List Item
description: A single item in a list.
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

| Name             | Description                                                | Type                                                                 | Inherited from          | `JSON-LD @id`                                              | Aliases                                                                                   |
| ---------------- | ---------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- | ---------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `id`             | The identifier for this item.                              | [`String`](./string.md)                                              | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)                       | -                                                                                         |
| `alternateNames` | Alternate names (aliases) for the item.                    | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   | [`schema:alternateName`](https://schema.org/alternateName) | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` |
| `description`    | A description of the item.                                 | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:description`](https://schema.org/description)     | -                                                                                         |
| `identifiers`    | Any kind of identifier for any kind of Thing.              | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   | [`schema:identifier`](https://schema.org/identifier)       | `identifier`                                                                              |
| `images`         | Images of the item.                                        | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   | [`schema:image`](https://schema.org/image)                 | `image`                                                                                   |
| `name`           | The name of the item.                                      | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:name`](https://schema.org/name)                   | -                                                                                         |
| `url`            | The URL of the item.                                       | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:url`](https://schema.org/url)                     | -                                                                                         |
| `content`        | The content of the list item.                              | [`Block`](./block.md)*                                               | -                       | `stencila:content`                                         | -                                                                                         |
| `item`           | The item represented by this list item.                    | [`Node`](./node.md)                                                  | -                       | [`schema:item`](https://schema.org/item)                   | -                                                                                         |
| `isChecked`      | A flag to indicate if this list item is checked.           | [`Boolean`](./boolean.md)                                            | -                       | `stencila:isChecked`                                       | `is-checked`, `is_checked`                                                                |
| `position`       | The position of the item in a series or sequence of items. | [`Integer`](./integer.md)                                            | -                       | [`schema:position`](https://schema.org/position)           | -                                                                                         |

# Related

The `ListItem` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Formats

The `ListItem` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support                                                                                                      | Notes |
| ------------------------------------------------ | ------------ | ------------ | ------------------------------------------------------------------------------------------------------------ | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |                                                                                                              |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              | Encoded as [`<li>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li)                            |
| [JATS](../formats/jats.md)                       | 🔷 Low loss   |              | Encoded as [`<list-item>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/list-item.html) |
| [Markdown](../formats/md.md)                     | 🔷 Low loss   | 🔷 Low loss   | Encoded using implemented function                                                                           |
| [Stencila Markdown](../formats/smd.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [Quarto Markdown](../formats/qmd.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [MyST Markdown](../formats/myst.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [LLM Markdown](../formats/llmd.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |                                                                                                              |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |                                                                                                              |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                              |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |                                                                                                              |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |                                                                                                              |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                              |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                              |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |                                                                                                              |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |                                                                                                              |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |                                                                                                              |
| [CSL-JSON](../formats/csl.md)                    |              |              |                                                                                                              |
| [Citation File Format](../formats/cff.md)        |              |              |                                                                                                              |
| [CSV](../formats/csv.md)                         |              |              |                                                                                                              |
| [TSV](../formats/tsv.md)                         |              |              |                                                                                                              |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |                                                                                                              |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |                                                                                                              |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |                                                                                                              |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |                                                                                                              |
| [Directory](../formats/directory.md)             |              |              |                                                                                                              |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |                                                                                                              |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |                                                                                                              |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |                                                                                                              |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |                                                                                                              |
| [Email HTML](../formats/email.html.md)           |              |              |                                                                                                              |
| [MJML](../formats/mjml.md)                       |              |              |                                                                                                              |

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
