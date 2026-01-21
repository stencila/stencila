---
title: Contact Point
description: A contact point, usually within an organization.
---

This is an implementation of schema.org [`ContactPoint`](https://schema.org/ContactPoint). It extends schema.org `ContactPoint` by, adding a `content` property which must be an array of [`Block`](./block.md), as well as the properties added by [`CreativeWork`](./creative-work.md) which it extends.
`ContactPoint` is analogous, and structurally similar to, the JATS XML [`<corresp>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/corresp.html) element and the HTML5 [`<address>`](https://dev.w3.org/html5/html-author/#the-address-element) element.

# Properties

The `ContactPoint` type has these properties:

| Name                 | Description                                                                                                    | Type                                                                 | Inherited from          |
| -------------------- | -------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `id`                 | The identifier for this item.                                                                                  | [`String`](./string.md)                                              | [`Entity`](./entity.md) |
| `alternateNames`     | Alternate names (aliases) for the item.                                                                        | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`        | A description of the item.                                                                                     | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`        | Any kind of identifier for any kind of Thing.                                                                  | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`             | Images of the item.                                                                                            | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`               | The name of the item.                                                                                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`                | The URL of the item.                                                                                           | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `emails`             | Email address for correspondence.                                                                              | [`String`](./string.md)*                                             | -                       |
| `telephoneNumbers`   | Telephone numbers for the contact point.                                                                       | [`String`](./string.md)*                                             | -                       |
| `availableLanguages` | Languages (human not programming) in which it is possible to communicate with the organization/department etc. | [`String`](./string.md)*                                             | -                       |

# Related

The `ContactPoint` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: [`PostalAddress`](./postal-address.md)

# Bindings

The `ContactPoint` type is represented in:

- [JSON-LD](https://stencila.org/ContactPoint.jsonld)
- [JSON Schema](https://stencila.org/ContactPoint.schema.json)
- Python class [`ContactPoint`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/contact_point.py)
- Rust struct [`ContactPoint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/contact_point.rs)
- TypeScript class [`ContactPoint`](https://github.com/stencila/stencila/blob/main/ts/src/types/ContactPoint.ts)

# Source

This documentation was generated from [`ContactPoint.yaml`](https://github.com/stencila/stencila/blob/main/schema/ContactPoint.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
