---
title: Postal Address
description: A physical mailing address.
config:
  publish:
    ghost:
      type: post
      slug: postal-address
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Other
---

# Properties

The `PostalAddress` type has these properties:

| Name                  | Description                                                                                                    | Type                                                                                                                                                       | Inherited from                                                                  | `JSON-LD @id`                                                          | Aliases                                                                                                          |
| --------------------- | -------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ---------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `id`                  | The identifier for this item.                                                                                  | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)              | [`schema:id`](https://schema.org/id)                                   | -                                                                                                                |
| `alternateNames`      | Alternate names (aliases) for the item.                                                                        | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)                | [`schema:alternateName`](https://schema.org/alternateName)             | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name`                        |
| `description`         | A description of the item.                                                                                     | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)                | [`schema:description`](https://schema.org/description)                 | -                                                                                                                |
| `identifiers`         | Any kind of identifier for any kind of Thing.                                                                  | ([`PropertyValue`](https://stencila.ghost.io/docs/reference/schema/property-value) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string))* | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)                | [`schema:identifier`](https://schema.org/identifier)                   | `identifier`                                                                                                     |
| `images`              | Images of the item.                                                                                            | [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)*                                                                             | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)                | [`schema:image`](https://schema.org/image)                             | `image`                                                                                                          |
| `name`                | The name of the item.                                                                                          | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)                | [`schema:name`](https://schema.org/name)                               | -                                                                                                                |
| `url`                 | The URL of the item.                                                                                           | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)                | [`schema:url`](https://schema.org/url)                                 | -                                                                                                                |
| `emails`              | Email address for correspondence.                                                                              | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | [`ContactPoint`](https://stencila.ghost.io/docs/reference/schema/contact-point) | [`schema:email`](https://schema.org/email)                             | `email`                                                                                                          |
| `telephoneNumbers`    | Telephone numbers for the contact point.                                                                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | [`ContactPoint`](https://stencila.ghost.io/docs/reference/schema/contact-point) | [`schema:telephone`](https://schema.org/telephone)                     | `telephone`, `telephone-numbers`, `telephone_numbers`, `telephoneNumber`, `telephone-number`, `telephone_number` |
| `availableLanguages`  | Languages (human not programming) in which it is possible to communicate with the organization/department etc. | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | [`ContactPoint`](https://stencila.ghost.io/docs/reference/schema/contact-point) | [`schema:availableLanguage`](https://schema.org/availableLanguage)     | `available-languages`, `available_languages`, `availableLanguage`, `available-language`, `available_language`    |
| `streetAddress`       | The street address.                                                                                            | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                               | [`schema:streetAddress`](https://schema.org/streetAddress)             | `street-address`, `street_address`                                                                               |
| `postOfficeBoxNumber` | The post office box number.                                                                                    | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                               | [`schema:postOfficeBoxNumber`](https://schema.org/postOfficeBoxNumber) | `post-office-box-number`, `post_office_box_number`                                                               |
| `addressLocality`     | The locality in which the street address is, and which is in the region.                                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                               | [`schema:addressLocality`](https://schema.org/addressLocality)         | `address-locality`, `address_locality`                                                                           |
| `addressRegion`       | The region in which the locality is, and which is in the country.                                              | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                               | [`schema:addressRegion`](https://schema.org/addressRegion)             | `address-region`, `address_region`                                                                               |
| `postalCode`          | The postal code.                                                                                               | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                               | [`schema:postalCode`](https://schema.org/postalCode)                   | `postal-code`, `postal_code`                                                                                     |
| `addressCountry`      | The country.                                                                                                   | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | -                                                                               | [`schema:addressCountry`](https://schema.org/addressCountry)           | `address-country`, `address_country`                                                                             |

# Related

The `PostalAddress` type is related to these types:

- Parents: [`ContactPoint`](https://stencila.ghost.io/docs/reference/schema/contact-point)
- Children: none

# Formats

The `PostalAddress` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                               | Encoding     | Decoding   | Support                                                                                                  | Notes |
| ------------------------------------------------------------------------------------ | ------------ | ---------- | -------------------------------------------------------------------------------------------------------- | ----- |
| [DOM HTML](https://stencila.ghost.io/docs/reference/formats/dom.html)                | 🟢 No loss    |            |                                                                                                          |
| [HTML](https://stencila.ghost.io/docs/reference/formats/html)                        | 🔷 Low loss   |            |                                                                                                          |
| [JATS](https://stencila.ghost.io/docs/reference/formats/jats)                        | 🔷 Low loss   |            | Encoded as [`<address>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/address.html) |
| [Markdown](https://stencila.ghost.io/docs/reference/formats/md)                      | ⚠️ High loss |            |                                                                                                          |
| [Stencila Markdown](https://stencila.ghost.io/docs/reference/formats/smd)            | ⚠️ High loss |            |                                                                                                          |
| [Quarto Markdown](https://stencila.ghost.io/docs/reference/formats/qmd)              | ⚠️ High loss |            |                                                                                                          |
| [MyST Markdown](https://stencila.ghost.io/docs/reference/formats/myst)               | ⚠️ High loss |            |                                                                                                          |
| [LLM Markdown](https://stencila.ghost.io/docs/reference/formats/llmd)                | ⚠️ High loss |            |                                                                                                          |
| [LaTeX](https://stencila.ghost.io/docs/reference/formats/latex)                      | 🔷 Low loss   | 🔷 Low loss |                                                                                                          |
| [PDF](https://stencila.ghost.io/docs/reference/formats/pdf)                          | 🔷 Low loss   |            |                                                                                                          |
| [Plain text](https://stencila.ghost.io/docs/reference/formats/text)                  | ⚠️ High loss |            |                                                                                                          |
| [IPYNB](https://stencila.ghost.io/docs/reference/formats/ipynb)                      | 🔷 Low loss   | 🔷 Low loss |                                                                                                          |
| [Microsoft Word DOCX](https://stencila.ghost.io/docs/reference/formats/docx)         | 🔷 Low loss   | 🔷 Low loss |                                                                                                          |
| [OpenDocument ODT](https://stencila.ghost.io/docs/reference/formats/odt)             | 🔷 Low loss   | 🔷 Low loss |                                                                                                          |
| [TeX](https://stencila.ghost.io/docs/reference/formats/tex)                          | 🔷 Low loss   | 🔷 Low loss |                                                                                                          |
| [JSON](https://stencila.ghost.io/docs/reference/formats/json)                        | 🟢 No loss    | 🟢 No loss  |                                                                                                          |
| [JSON+Zip](https://stencila.ghost.io/docs/reference/formats/json.zip)                | 🟢 No loss    | 🟢 No loss  |                                                                                                          |
| [JSON5](https://stencila.ghost.io/docs/reference/formats/json5)                      | 🟢 No loss    | 🟢 No loss  |                                                                                                          |
| [JSON-LD](https://stencila.ghost.io/docs/reference/formats/jsonld)                   | 🟢 No loss    | 🟢 No loss  |                                                                                                          |
| [CBOR](https://stencila.ghost.io/docs/reference/formats/cbor)                        | 🟢 No loss    | 🟢 No loss  |                                                                                                          |
| [CBOR+Zstd](https://stencila.ghost.io/docs/reference/formats/cbor.zstd)              | 🟢 No loss    | 🟢 No loss  |                                                                                                          |
| [YAML](https://stencila.ghost.io/docs/reference/formats/yaml)                        | 🟢 No loss    | 🟢 No loss  |                                                                                                          |
| [Lexical JSON](https://stencila.ghost.io/docs/reference/formats/lexical)             | 🔷 Low loss   | 🔷 Low loss |                                                                                                          |
| [Koenig JSON](https://stencila.ghost.io/docs/reference/formats/koenig)               | 🔷 Low loss   | 🔷 Low loss |                                                                                                          |
| [Pandoc AST](https://stencila.ghost.io/docs/reference/formats/pandoc)                | 🔷 Low loss   | 🔷 Low loss |                                                                                                          |
| [Directory](https://stencila.ghost.io/docs/reference/formats/directory)              |              |            |                                                                                                          |
| [Stencila Web Bundle](https://stencila.ghost.io/docs/reference/formats/swb)          |              |            |                                                                                                          |
| [PubMed Central OA Package](https://stencila.ghost.io/docs/reference/formats/pmcoap) |              | 🔷 Low loss |                                                                                                          |
| [Debug](https://stencila.ghost.io/docs/reference/formats/debug)                      | 🔷 Low loss   |            |                                                                                                          |

# Bindings

The `PostalAddress` type is represented in:

- [JSON-LD](https://stencila.org/PostalAddress.jsonld)
- [JSON Schema](https://stencila.org/PostalAddress.schema.json)
- Python class [`PostalAddress`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/postal_address.py)
- Rust struct [`PostalAddress`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/postal_address.rs)
- TypeScript class [`PostalAddress`](https://github.com/stencila/stencila/blob/main/ts/src/types/PostalAddress.ts)

# Source

This documentation was generated from [`PostalAddress.yaml`](https://github.com/stencila/stencila/blob/main/schema/PostalAddress.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
