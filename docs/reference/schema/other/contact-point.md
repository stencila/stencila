---
title: Contact Point
description: A contact point, usually within an organization.
config:
  publish:
    ghost:
      type: post
      slug: contact-point
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Other
---

This is an implementation of schema.org [`ContactPoint`](https://schema.org/ContactPoint). It extends schema.org `ContactPoint` by, adding a `content` property which must be an array of [`Block`](./Block), as well as the properties added by [`CreativeWork`](./CreativeWork) which it extends.
`ContactPoint` is analogous, and structurally similar to, the JATS XML [`<corresp>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/corresp.html) element and the HTML5 [`<address>`](https://dev.w3.org/html5/html-author/#the-address-element) element.

# Properties

The `ContactPoint` type has these properties:

| Name                 | Description                                                                                                    | Type                                                                                                                                                       | Inherited from                                                     | `JSON-LD @id`                                                      | Aliases                                                                                                          |
| -------------------- | -------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------- |
| `id`                 | The identifier for this item.                                                                                  | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id)                               | -                                                                                                                |
| `alternateNames`     | Alternate names (aliases) for the item.                                                                        | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:alternateName`](https://schema.org/alternateName)         | `alternate-names`, `alternate_names`, `alternateName`, `alternate-name`, `alternate_name`                        |
| `description`        | A description of the item.                                                                                     | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:description`](https://schema.org/description)             | -                                                                                                                |
| `identifiers`        | Any kind of identifier for any kind of Thing.                                                                  | ([`PropertyValue`](https://stencila.ghost.io/docs/reference/schema/property-value) \| [`String`](https://stencila.ghost.io/docs/reference/schema/string))* | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:identifier`](https://schema.org/identifier)               | `identifier`                                                                                                     |
| `images`             | Images of the item.                                                                                            | [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)*                                                                             | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:image`](https://schema.org/image)                         | `image`                                                                                                          |
| `name`               | The name of the item.                                                                                          | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:name`](https://schema.org/name)                           | -                                                                                                                |
| `url`                | The URL of the item.                                                                                           | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                                                                                         | [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)   | [`schema:url`](https://schema.org/url)                             | -                                                                                                                |
| `emails`             | Email address for correspondence.                                                                              | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | -                                                                  | [`schema:email`](https://schema.org/email)                         | `email`                                                                                                          |
| `telephoneNumbers`   | Telephone numbers for the contact point.                                                                       | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | -                                                                  | [`schema:telephone`](https://schema.org/telephone)                 | `telephone`, `telephone-numbers`, `telephone_numbers`, `telephoneNumber`, `telephone-number`, `telephone_number` |
| `availableLanguages` | Languages (human not programming) in which it is possible to communicate with the organization/department etc. | [`String`](https://stencila.ghost.io/docs/reference/schema/string)*                                                                                        | -                                                                  | [`schema:availableLanguage`](https://schema.org/availableLanguage) | `available-languages`, `available_languages`, `availableLanguage`, `available-language`, `available_language`    |

# Related

The `ContactPoint` type is related to these types:

- Parents: [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing)
- Children: [`PostalAddress`](https://stencila.ghost.io/docs/reference/schema/postal-address)

# Formats

The `ContactPoint` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `ContactPoint` type is represented in:

- [JSON-LD](https://stencila.org/ContactPoint.jsonld)
- [JSON Schema](https://stencila.org/ContactPoint.schema.json)
- Python class [`ContactPoint`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/contact_point.py)
- Rust struct [`ContactPoint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/contact_point.rs)
- TypeScript class [`ContactPoint`](https://github.com/stencila/stencila/blob/main/ts/src/types/ContactPoint.ts)

# Source

This documentation was generated from [`ContactPoint.yaml`](https://github.com/stencila/stencila/blob/main/schema/ContactPoint.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
