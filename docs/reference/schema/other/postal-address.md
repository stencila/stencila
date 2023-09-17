---
title:
- type: Text
  value: PostalAddress
---

# Postal Address

**A physical mailing address.**

**`@id`**: [`schema:PostalAddress`](https://schema.org/PostalAddress)

## Properties

The `PostalAddress` type has these properties:

| Name                | `@id`                                                                  | Type                                                                                                                                                       | Description                                                                                                     | Inherited from                                                                     |
| ------------------- | ---------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| id                  | [`schema:id`](https://schema.org/id)                                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The identifier for this item                                                                                    | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)                |
| alternateNames      | [`schema:alternateName`](https://schema.org/alternateName)             | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Alternate names (aliases) for the item.                                                                         | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                  |
| description         | [`schema:description`](https://schema.org/description)                 | [`Block`](https://stencila.dev/docs/reference/schema/prose/block)*                                                                                         | A description of the item.                                                                                      | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                  |
| identifiers         | [`schema:identifier`](https://schema.org/identifier)                   | ([`PropertyValue`](https://stencila.dev/docs/reference/schema/other/property-value) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))* | Any kind of identifier for any kind of Thing.                                                                   | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                  |
| images              | [`schema:image`](https://schema.org/image)                             | ([`ImageObject`](https://stencila.dev/docs/reference/schema/works/image-object) \| [`String`](https://stencila.dev/docs/reference/schema/data/string))*    | Images of the item.                                                                                             | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                  |
| name                | [`schema:name`](https://schema.org/name)                               | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The name of the item.                                                                                           | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                  |
| url                 | [`schema:url`](https://schema.org/url)                                 | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The URL of the item.                                                                                            | [`Thing`](https://stencila.dev/docs/reference/schema/other/thing)                  |
| emails              | [`schema:email`](https://schema.org/email)                             | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Email address for correspondence.                                                                               | [`ContactPoint`](https://stencila.dev/docs/reference/schema/other/contact-point)   |
| telephoneNumbers    | [`schema:telephone`](https://schema.org/telephone)                     | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Telephone numbers for the contact point.                                                                        | [`ContactPoint`](https://stencila.dev/docs/reference/schema/other/contact-point)   |
| availableLanguages  | [`schema:availableLanguage`](https://schema.org/availableLanguage)     | [`String`](https://stencila.dev/docs/reference/schema/data/string)*                                                                                        | Languages (human not programming) in which it is possible to communicate with the organization/department etc.  | [`ContactPoint`](https://stencila.dev/docs/reference/schema/other/contact-point)   |
| streetAddress       | [`schema:streetAddress`](https://schema.org/streetAddress)             | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The street address.                                                                                             | [`PostalAddress`](https://stencila.dev/docs/reference/schema/other/postal-address) |
| postOfficeBoxNumber | [`schema:postOfficeBoxNumber`](https://schema.org/postOfficeBoxNumber) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The post office box number.                                                                                     | [`PostalAddress`](https://stencila.dev/docs/reference/schema/other/postal-address) |
| addressLocality     | [`schema:addressLocality`](https://schema.org/addressLocality)         | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The locality in which the street address is, and which is in the region.                                        | [`PostalAddress`](https://stencila.dev/docs/reference/schema/other/postal-address) |
| addressRegion       | [`schema:addressRegion`](https://schema.org/addressRegion)             | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The region in which the locality is, and which is in the country.                                               | [`PostalAddress`](https://stencila.dev/docs/reference/schema/other/postal-address) |
| postalCode          | [`schema:postalCode`](https://schema.org/postalCode)                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The postal code.                                                                                                | [`PostalAddress`](https://stencila.dev/docs/reference/schema/other/postal-address) |
| addressCountry      | [`schema:addressCountry`](https://schema.org/addressCountry)           | [`String`](https://stencila.dev/docs/reference/schema/data/string)                                                                                         | The country.                                                                                                    | [`PostalAddress`](https://stencila.dev/docs/reference/schema/other/postal-address) |

## Related

The `PostalAddress` type is related to these types:

- Parents: [`ContactPoint`](https://stencila.dev/docs/reference/schema/other/contact-point)
- Children: none

## Formats

The `PostalAddress` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes                                                                                                   |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ------------------------------------------------------------------------------------------------------- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |                                                                                                         |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    | Encoded to tag [`<address>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/address) |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |                                                                                                         |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |                                                                                                         |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                         |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                         |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |                                                                                                         |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |                                                                                                         |

## Bindings

The `PostalAddress` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/PostalAddress.jsonld)
- [JSON Schema](https://stencila.dev/PostalAddress.schema.json)
- Python class [`PostalAddress`](https://github.com/stencila/stencila/blob/main/python/stencila/types/postal_address.py)
- Rust struct [`PostalAddress`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/postal_address.rs)
- TypeScript class [`PostalAddress`](https://github.com/stencila/stencila/blob/main/typescript/src/types/PostalAddress.ts)

## Source

This documentation was generated from [`PostalAddress.yaml`](https://github.com/stencila/stencila/blob/main/schema/PostalAddress.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).