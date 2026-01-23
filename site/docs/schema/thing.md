---
title: Thing
description: The most generic type of item.
---

# Properties

The `Thing` type has these properties:

| Name             | Description                                   | Type                                                                 | Inherited from          |
| ---------------- | --------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `id`             | The identifier for this item.                 | [`String`](./string.md)                                              | [`Entity`](./entity.md) |
| `alternateNames` | Alternate names (aliases) for the item.       | [`String`](./string.md)*                                             | -                       |
| `description`    | A description of the item.                    | [`String`](./string.md)                                              | -                       |
| `identifiers`    | Any kind of identifier for any kind of Thing. | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | -                       |
| `images`         | Images of the item.                           | [`ImageObject`](./image-object.md)*                                  | -                       |
| `name`           | The name of the item.                         | [`String`](./string.md)                                              | -                       |
| `url`            | The URL of the item.                          | [`String`](./string.md)                                              | -                       |

# Related

The `Thing` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: [`Brand`](./brand.md), [`ContactPoint`](./contact-point.md), [`CreativeWork`](./creative-work.md), [`DefinedTerm`](./defined-term.md), [`Enumeration`](./enumeration.md), [`Grant`](./grant.md), [`ListItem`](./list-item.md), [`Organization`](./organization.md), [`Person`](./person.md), [`Product`](./product.md), [`PropertyValue`](./property-value.md)

# Bindings

The `Thing` type is represented in:

- [JSON-LD](https://stencila.org/Thing.jsonld)
- [JSON Schema](https://stencila.org/Thing.schema.json)
- Python class [`Thing`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Thing`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/thing.rs)
- TypeScript class [`Thing`](https://github.com/stencila/stencila/blob/main/ts/src/types/Thing.ts)

***

This documentation was generated from [`Thing.yaml`](https://github.com/stencila/stencila/blob/main/schema/Thing.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
