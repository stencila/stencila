---
title: Contact Point
description: A contact point, usually within an organization.
---

This is an implementation of schema.org [`ContactPoint`](https://schema.org/ContactPoint), adapted in Stencila Schema for structured contact metadata.
It is used for correspondence and contact details associated with people, organizations, and related entities, while fitting into Stencila's broader node model.
Key properties include `emails`, `telephoneNumbers`, and `availableLanguages`.

# Analogues

The following external types, elements, or nodes are similar to a `ContactPoint`:

- schema.org [`ContactPoint`](https://schema.org/ContactPoint)

# Properties

The `ContactPoint` type has these properties:

| Name                 | Description                                                                                                    | Type                                                                 | Inherited from          |
| -------------------- | -------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `emails`             | Email address for correspondence.                                                                              | [`String`](./string.md)*                                             | -                       |
| `telephoneNumbers`   | Telephone numbers for the contact point.                                                                       | [`String`](./string.md)*                                             | -                       |
| `availableLanguages` | Languages (human not programming) in which it is possible to communicate with the organization/department etc. | [`String`](./string.md)*                                             | -                       |
| `alternateNames`     | Alternate names (aliases) for the item.                                                                        | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`        | A description of the item.                                                                                     | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`        | Any kind of identifier for any kind of Thing.                                                                  | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`             | Images of the item.                                                                                            | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`               | The name of the item.                                                                                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`                | The URL of the item.                                                                                           | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `id`                 | The identifier for this item.                                                                                  | [`String`](./string.md)                                              | [`Entity`](./entity.md) |

# Related

The `ContactPoint` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: [`PostalAddress`](./postal-address.md)

# Bindings

The `ContactPoint` type is represented in:

- [JSON-LD](https://stencila.org/ContactPoint.jsonld)
- [JSON Schema](https://stencila.org/ContactPoint.schema.json)
- Python class [`ContactPoint`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ContactPoint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/contact_point.rs)
- TypeScript class [`ContactPoint`](https://github.com/stencila/stencila/blob/main/ts/src/types/ContactPoint.ts)

***

This documentation was generated from [`ContactPoint.yaml`](https://github.com/stencila/stencila/blob/main/schema/ContactPoint.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
