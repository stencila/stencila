---
title: Property Value
description: A property-value pair.
---

Always use specific properties when they exist and you can populate them.
Using `PropertyValue` as a substitute will typically not convey as much semantic
information as using the specific property.

Most of the time a `PropertyValue` node will need a `value` property
(e.g. most of the examples on https://schema.org/PropertyValue have one)
so this schema make that property required.

This type is mainly provided for use in `Thing.identifiers` (see the notes there).


# Properties

The `PropertyValue` type has these properties:

| Name             | Description                                                                    | Type                                                                 | Inherited from          | `JSON-LD @id`                                              | Aliases                                                                                   |
| ---------------- | ------------------------------------------------------------------------------ | -------------------------------------------------------------------- | ----------------------- | ---------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `id`             | The identifier for this item.                                                  | [`String`](./string.md)                                              | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)                       | -                                                                                         |
| `alternateNames` | Alternate names (aliases) for the item.                                        | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   | [`schema:alternateName`](https://schema.org/alternateName) | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` |
| `description`    | A description of the item.                                                     | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:description`](https://schema.org/description)     | -                                                                                         |
| `identifiers`    | Any kind of identifier for any kind of Thing.                                  | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   | [`schema:identifier`](https://schema.org/identifier)       | `identifier`                                                                              |
| `images`         | Images of the item.                                                            | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   | [`schema:image`](https://schema.org/image)                 | `image`                                                                                   |
| `name`           | The name of the item.                                                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:name`](https://schema.org/name)                   | -                                                                                         |
| `url`            | The URL of the item.                                                           | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:url`](https://schema.org/url)                     | -                                                                                         |
| `propertyID`     | A commonly used identifier for the characteristic represented by the property. | [`String`](./string.md)                                              | -                       | [`schema:propertyID`](https://schema.org/propertyID)       | `property-id`, `property_id`                                                              |
| `value`          | The value of the property.                                                     | [`Primitive`](./primitive.md)                                        | -                       | [`schema:value`](https://schema.org/value)                 | -                                                                                         |

# Related

The `PropertyValue` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Formats

The `PropertyValue` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                           | Encoding     | Decoding     | Support | Notes |
| ------------------------------------------------ | ------------ | ------------ | ------- | ----- |
| [DOM HTML](../formats/dom.html.md)               | 🟢 No loss    |              |         |
| [HTML](../formats/html.md)                       | 🟢 No loss    |              |         |
| [JATS](../formats/jats.md)                       |              |              |         |
| [Markdown](../formats/md.md)                     | ⚠️ High loss |              |         |
| [Stencila Markdown](../formats/smd.md)           | ⚠️ High loss |              |         |
| [Quarto Markdown](../formats/qmd.md)             | ⚠️ High loss |              |         |
| [MyST Markdown](../formats/myst.md)              | ⚠️ High loss |              |         |
| [LLM Markdown](../formats/llmd.md)               | ⚠️ High loss |              |         |
| [LaTeX](../formats/latex.md)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [R+LaTeX](../formats/rnw.md)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [PDF](../formats/pdf.md)                         | ⚠️ High loss | ⚠️ High loss |         |
| [Plain text](../formats/text.md)                 | ⚠️ High loss |              |         |
| [IPYNB](../formats/ipynb.md)                     | 🔷 Low loss   | 🔷 Low loss   |         |
| [Microsoft Word](../formats/docx.md)             | 🔷 Low loss   | 🔷 Low loss   |         |
| [OpenDocument Text](../formats/odt.md)           | 🔷 Low loss   | 🔷 Low loss   |         |
| [TeX](../formats/tex.md)                         | 🔷 Low loss   | 🔷 Low loss   |         |
| [JSON](../formats/json.md)                       | 🟢 No loss    | 🟢 No loss    |         |
| [JSON+Zip](../formats/json.zip.md)               | 🟢 No loss    | 🟢 No loss    |         |
| [JSON5](../formats/json5.md)                     | 🟢 No loss    | 🟢 No loss    |         |
| [JSON-LD](../formats/jsonld.md)                  | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR](../formats/cbor.md)                       | 🟢 No loss    | 🟢 No loss    |         |
| [CBOR+Zstd](../formats/czst.md)                  | 🟢 No loss    | 🟢 No loss    |         |
| [YAML](../formats/yaml.md)                       | 🟢 No loss    | 🟢 No loss    |         |
| [Lexical JSON](../formats/lexical.md)            | 🔷 Low loss   | 🔷 Low loss   |         |
| [Koenig JSON](../formats/koenig.md)              | 🔷 Low loss   | 🔷 Low loss   |         |
| [Pandoc AST](../formats/pandoc.md)               | 🔷 Low loss   | 🔷 Low loss   |         |
| [CSL-JSON](../formats/csl.md)                    |              |              |         |
| [Citation File Format](../formats/cff.md)        |              |              |         |
| [CSV](../formats/csv.md)                         |              |              |         |
| [TSV](../formats/tsv.md)                         |              |              |         |
| [Microsoft Excel](../formats/xlsx.md)            |              |              |         |
| [Microsoft Excel (XLS)](../formats/xls.md)       |              |              |         |
| [OpenDocument Spreadsheet](../formats/ods.md)    |              |              |         |
| [PNG](../formats/png.md)                         | ⚠️ High loss |              |         |
| [Directory](../formats/directory.md)             |              |              |         |
| [Stencila Web Bundle](../formats/swb.md)         |              |              |         |
| [Meca](../formats/meca.md)                       |              | 🔷 Low loss   |         |
| [PubMed Central OA Package](../formats/pmcoa.md) |              |              |         |
| [Debug](../formats/debug.md)                     | 🔷 Low loss   |              |         |
| [Email HTML](../formats/email.html.md)           |              |              |         |
| [MJML](../formats/mjml.md)                       |              |              |         |

# Bindings

The `PropertyValue` type is represented in:

- [JSON-LD](https://stencila.org/PropertyValue.jsonld)
- [JSON Schema](https://stencila.org/PropertyValue.schema.json)
- Python class [`PropertyValue`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/property_value.py)
- Rust struct [`PropertyValue`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/property_value.rs)
- TypeScript class [`PropertyValue`](https://github.com/stencila/stencila/blob/main/ts/src/types/PropertyValue.ts)

# Source

This documentation was generated from [`PropertyValue.yaml`](https://github.com/stencila/stencila/blob/main/schema/PropertyValue.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
