---
title: Brand
description: A brand used by an organization or person for labeling a product, product group, or similar.
config:
  publish:
    ghost:
      type: post
      slug: brand
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Other
---

# Properties

The `Brand` type has these properties:

| Name             | Description                                   | Type                                                                                                                                                       | Inherited from                                                     | `JSON-LD @id`                                              | Aliases                                                                                   |
| ---------------- | --------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ---------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `id`             | The identifier for this item.                 | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                       | -                                                                                         |
| `alternateNames` | Alternate names (aliases) for the item.       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:alternateName`](https://schema.org/alternateName) | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name` |
| `description`    | A description of the item.                    | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:description`](https://schema.org/description)     | -                                                                                         |
| `identifiers`    | Any kind of identifier for any kind of Thing. | ([`PropertyValue`](https://stencila.ghost.io/docs/reference/schema/property-value) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string))* | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:identifier`](https://schema.org/identifier)       | `identifier`                                                                              |
| `images`         | Images of the item.                           | [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)*                                                                             | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:image`](https://schema.org/image)                 | `image`                                                                                   |
| `name`           | The name of the item.                         | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:name`](https://schema.org/name)                   | -                                                                                         |
| `url`            | The URL of the item.                          | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:url`](https://schema.org/url)                     | -                                                                                         |
| `logo`           | A logo associated with the brand.             | [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)                                                                              | -                                                                  | [`schema:logo`](https://schema.org/logo)                   | -                                                                                         |
| `reviews`        | Reviews of the brand.                         | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | -                                                                  | [`schema:review`](https://schema.org/review)               | `review`                                                                                  |

# Related

The `Brand` type is related to these types:

- Parents: [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)
- Children: none

# Formats

The `Brand` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                               | Encoding     | Decoding   | Support | Notes |
| ------------------------------------------------------------------------------------ | ------------ | ---------- | ------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)                | 🟢 No loss    |            |         |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                        | 🔷 Low loss   |            |         |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                        |              |            |         |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                      | ⚠️ High loss |            |         |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)            | ⚠️ High loss |            |         |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)              | ⚠️ High loss |            |         |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)               | ⚠️ High loss |            |         |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)                | ⚠️ High loss |            |         |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                      | 🔷 Low loss   | 🔷 Low loss |         |
| [R+LaTeX](https://stencila.ghost.io/docs/reference/formats/rnw)                      | 🔷 Low loss   | 🔷 Low loss |         |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                          | ⚠️ High loss |            |         |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                  | ⚠️ High loss |            |         |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                      | 🔷 Low loss   | 🔷 Low loss |         |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)         | 🔷 Low loss   | 🔷 Low loss |         |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)             | 🔷 Low loss   | 🔷 Low loss |         |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                          | 🔷 Low loss   | 🔷 Low loss |         |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                        | 🟢 No loss    | 🟢 No loss  |         |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)                | 🟢 No loss    | 🟢 No loss  |         |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                      | 🟢 No loss    | 🟢 No loss  |         |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                   | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                        | 🟢 No loss    | 🟢 No loss  |         |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)              | 🟢 No loss    | 🟢 No loss  |         |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                        | 🟢 No loss    | 🟢 No loss  |         |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)             | 🔷 Low loss   | 🔷 Low loss |         |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)               | 🔷 Low loss   | 🔷 Low loss |         |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)                | 🔷 Low loss   | 🔷 Low loss |         |
| [PNG](https://stencila.ghost.io/docs/reference/formats/png)                          | ⚠️ High loss |            |         |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)              |              |            |         |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)          |              |            |         |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoap) |              | 🔷 Low loss |         |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                      | 🔷 Low loss   |            |         |

# Bindings

The `Brand` type is represented in:

- [JSON-LD](https://stencila.org/Brand.jsonld)
- [JSON Schema](https://stencila.org/Brand.schema.json)
- Python class [`Brand`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/brand.py)
- Rust struct [`Brand`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/brand.rs)
- TypeScript class [`Brand`](https://github.com/stencila/stencila/blob/main/ts/src/types/Brand.ts)

# Source

This documentation was generated from [`Brand.yaml`](https://github.com/stencila/stencila/blob/main/schema/Brand.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
