---
title: Monetary Grant
description: A monetary grant.
config:
  publish:
    ghost:
      type: post
      slug: monetary-grant
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Other
---

# Properties

The `MonetaryGrant` type has these properties:

| Name             | Description                                                                                              | Type                                                                                                                                                       | Inherited from                                                     | `JSON-LD @id`                                              | Aliases                                                                                   |
| ---------------- | -------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ---------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `id`             | The identifier for this item.                                                                            | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                       | -                                                                                         |
| `alternateNames` | Alternate names (aliases) for the item.                                                                  | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:alternateName`](https://schema.org/alternateName) | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` |
| `description`    | A description of the item.                                                                               | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:description`](https://schema.org/description)     | -                                                                                         |
| `identifiers`    | Any kind of identifier for any kind of Thing.                                                            | ([`PropertyValue`](https://stencila.ghost.io/docs/reference/schema/property-value) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string))* | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:identifier`](https://schema.org/identifier)       | `identifier`                                                                              |
| `images`         | Images of the item.                                                                                      | [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)*                                                                             | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:image`](https://schema.org/image)                 | `image`                                                                                   |
| `name`           | The name of the item.                                                                                    | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:name`](https://schema.org/name)                   | -                                                                                         |
| `url`            | The URL of the item.                                                                                     | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:url`](https://schema.org/url)                     | -                                                                                         |
| `fundedItems`    | Indicates an item funded or sponsored through a Grant.                                                   | [`ThingType`](https://stencila.ghost.io/docs/reference/schema/thing-type)*                                                                                 | [`Grant`](https://stencila.ghost.io/docs/reference/schema/grant)   | [`schema:fundedItem`](https://schema.org/fundedItem)       | `funded-items`, `funded_items`, `fundedItem`, `funded-item`, `funded_item`                |
| `sponsors`       | A person or organization that supports a thing through a pledge, promise, or financial contribution.     | ([`Person`](https://stencila.ghost.io/docs/reference/schema/person) \| [`Organization`](https://stencila.ghost.io/docs/reference/schema/organization))*    | [`Grant`](https://stencila.ghost.io/docs/reference/schema/grant)   | [`schema:sponsor`](https://schema.org/sponsor)             | `sponsor`                                                                                 |
| `amounts`        | The amount of money.                                                                                     | [`Number`](https://stencila.ghost.io/docs/reference/schema/number)                                                                                         | -                                                                  | [`schema:amount`](https://schema.org/amount)               | -                                                                                         |
| `funders`        | A person or organization that supports (sponsors) something through some kind of financial contribution. | ([`Person`](https://stencila.ghost.io/docs/reference/schema/person) \| [`Organization`](https://stencila.ghost.io/docs/reference/schema/organization))*    | -                                                                  | [`schema:funder`](https://schema.org/funder)               | `funder`                                                                                  |

# Related

The `MonetaryGrant` type is related to these types:

- Parents: [`Grant`](https://stencila.ghost.io/docs/reference/schema/grant)
- Children: none

# Formats

The `MonetaryGrant` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                              | Encoding     | Decoding   | Support | Notes |
| ----------------------------------------------------------------------------------- | ------------ | ---------- | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)               | 🟢 No loss    |            |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                       | 🟢 No loss    |            |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                       |              |            |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                     | ⚠️ High loss |            |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)           | ⚠️ High loss |            |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)             | ⚠️ High loss |            |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)              | ⚠️ High loss |            |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)               | ⚠️ High loss |            |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                     | 🔷 Low loss   | 🔷 Low loss |         |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                     | 🔷 Low loss   | 🔷 Low loss |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                         | ⚠️ High loss |            |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                 | ⚠️ High loss |            |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                     | 🔷 Low loss   | 🔷 Low loss |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)        | 🔷 Low loss   | 🔷 Low loss |         |
| [Google Docs DOCX](https://stencila.ghost.io/docs/reference/formats/gdocx)          |              |            |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)            | 🔷 Low loss   | 🔷 Low loss |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                         | 🔷 Low loss   | 🔷 Low loss |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                       | 🟢 No loss    | 🟢 No loss  |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)               | 🟢 No loss    | 🟢 No loss  |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                     | 🟢 No loss    | 🟢 No loss  |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                  | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                       | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)             | 🟢 No loss    | 🟢 No loss  |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                       | 🟢 No loss    | 🟢 No loss  |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)            | 🔷 Low loss   | 🔷 Low loss |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)              | 🔷 Low loss   | 🔷 Low loss |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)               | 🔷 Low loss   | 🔷 Low loss |         |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                         | ⚠️ High loss |            |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)             |              |            |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)         |              |            |         |
| [Meca](https://stencila.ghost.io/docs/reference/formats/meca)                       |              | 🔷 Low loss |         |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoa) |              | 🔷 Low loss |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                     | 🔷 Low loss   |            |         |

# Bindings

The `MonetaryGrant` type is represented in:

- [JSON-LD](https://stencila.org/MonetaryGrant.jsonld)
- [JSON Schema](https://stencila.org/MonetaryGrant.schema.json)
- Python class [`MonetaryGrant`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/monetary_grant.py)
- Rust struct [`MonetaryGrant`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/monetary_grant.rs)
- TypeScript class [`MonetaryGrant`](https://github.com/stencila/stencila/blob/main/ts/src/types/MonetaryGrant.ts)

# Source

This documentation was generated from [`MonetaryGrant.yaml`](https://github.com/stencila/stencila/blob/main/schema/MonetaryGrant.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
