---
title: Grant
description: A grant, typically financial or otherwise quantifiable, of resources.
---

# Properties

The `Grant` type has these properties:

| Name             | Description                                                                                          | Type                                                                 | Inherited from          | `JSON-LD @id`                                              | Aliases                                                                                   |
| ---------------- | ---------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- | ---------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `id`             | The identifier for this item.                                                                        | [`String`](./string.md)                                              | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id)                       | -                                                                                         |
| `alternateNames` | Alternate names (aliases) for the item.                                                              | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   | [`schema:alternateName`](https://schema.org/alternateName) | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` |
| `description`    | A description of the item.                                                                           | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:description`](https://schema.org/description)     | -                                                                                         |
| `identifiers`    | Any kind of identifier for any kind of Thing.                                                        | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   | [`schema:identifier`](https://schema.org/identifier)       | `identifier`                                                                              |
| `images`         | Images of the item.                                                                                  | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   | [`schema:image`](https://schema.org/image)                 | `image`                                                                                   |
| `name`           | The name of the item.                                                                                | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:name`](https://schema.org/name)                   | -                                                                                         |
| `url`            | The URL of the item.                                                                                 | [`String`](./string.md)                                              | [`Thing`](./thing.md)   | [`schema:url`](https://schema.org/url)                     | -                                                                                         |
| `fundedItems`    | Indicates an item funded or sponsored through a Grant.                                               | [`ThingVariant`](./thing-variant.md)*                                | -                       | [`schema:fundedItem`](https://schema.org/fundedItem)       | `funded-items`, `funded_items`, `fundedItem`, `funded-item`, `funded_item`                |
| `sponsors`       | A person or organization that supports a thing through a pledge, promise, or financial contribution. | ([`Person`](./person.md) \| [`Organization`](./organization.md))*    | -                       | [`schema:sponsor`](https://schema.org/sponsor)             | `sponsor`                                                                                 |

# Related

The `Grant` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: [`MonetaryGrant`](./monetary-grant.md)

# Formats

The `Grant` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `Grant` type is represented in:

- [JSON-LD](https://stencila.org/Grant.jsonld)
- [JSON Schema](https://stencila.org/Grant.schema.json)
- Python class [`Grant`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/grant.py)
- Rust struct [`Grant`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/grant.rs)
- TypeScript class [`Grant`](https://github.com/stencila/stencila/blob/main/ts/src/types/Grant.ts)

# Source

This documentation was generated from [`Grant.yaml`](https://github.com/stencila/stencila/blob/main/schema/Grant.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
