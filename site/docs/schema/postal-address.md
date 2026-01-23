---
title: Postal Address
description: A physical mailing address.
---

# Properties

The `PostalAddress` type has these properties:

| Name                  | Description                                                                                                    | Type                                                                 | Inherited from                       |
| --------------------- | -------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ------------------------------------ |
| `id`                  | The identifier for this item.                                                                                  | [`String`](./string.md)                                              | [`Entity`](./entity.md)              |
| `alternateNames`      | Alternate names (aliases) for the item.                                                                        | [`String`](./string.md)*                                             | [`Thing`](./thing.md)                |
| `description`         | A description of the item.                                                                                     | [`String`](./string.md)                                              | [`Thing`](./thing.md)                |
| `identifiers`         | Any kind of identifier for any kind of Thing.                                                                  | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)                |
| `images`              | Images of the item.                                                                                            | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)                |
| `name`                | The name of the item.                                                                                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)                |
| `url`                 | The URL of the item.                                                                                           | [`String`](./string.md)                                              | [`Thing`](./thing.md)                |
| `emails`              | Email address for correspondence.                                                                              | [`String`](./string.md)*                                             | [`ContactPoint`](./contact-point.md) |
| `telephoneNumbers`    | Telephone numbers for the contact point.                                                                       | [`String`](./string.md)*                                             | [`ContactPoint`](./contact-point.md) |
| `availableLanguages`  | Languages (human not programming) in which it is possible to communicate with the organization/department etc. | [`String`](./string.md)*                                             | [`ContactPoint`](./contact-point.md) |
| `streetAddress`       | The street address.                                                                                            | [`String`](./string.md)                                              | -                                    |
| `postOfficeBoxNumber` | The post office box number.                                                                                    | [`String`](./string.md)                                              | -                                    |
| `addressLocality`     | The locality in which the street address is, and which is in the region.                                       | [`String`](./string.md)                                              | -                                    |
| `addressRegion`       | The region in which the locality is, and which is in the country.                                              | [`String`](./string.md)                                              | -                                    |
| `postalCode`          | The postal code.                                                                                               | [`String`](./string.md)                                              | -                                    |
| `addressCountry`      | The country.                                                                                                   | [`String`](./string.md)                                              | -                                    |

# Related

The `PostalAddress` type is related to these types:

- Parents: [`ContactPoint`](./contact-point.md)
- Children: none

# Bindings

The `PostalAddress` type is represented in:

- [JSON-LD](https://stencila.org/PostalAddress.jsonld)
- [JSON Schema](https://stencila.org/PostalAddress.schema.json)
- Python class [`PostalAddress`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`PostalAddress`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/postal_address.rs)
- TypeScript class [`PostalAddress`](https://github.com/stencila/stencila/blob/main/ts/src/types/PostalAddress.ts)

***

This documentation was generated from [`PostalAddress.yaml`](https://github.com/stencila/stencila/blob/main/schema/PostalAddress.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
