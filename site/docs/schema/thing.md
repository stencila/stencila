---
title: Thing
description: The most generic type of item.
---

This is an implementation of schema.org [`Thing`](https://schema.org/Thing).

In Stencila Schema it provides the common base for schema.org-derived entities
such as people, organizations, creative works, products, and related
metadata-bearing types. Stencila keeps the broadly interoperable schema.org
core while integrating it with the node and document infrastructure inherited
through [`Entity`](./entity.md).

Key properties include `name`, `alternateNames`, `description`, `identifiers`,
`images`, and `url`.


# Properties

The `Thing` type has these properties:

| Name             | Description                                   | Type                                                                 | Inherited from          |
| ---------------- | --------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `alternateNames` | Alternate names (aliases) for the item.       | [`String`](./string.md)*                                             | -                       |
| `description`    | A description of the item.                    | [`String`](./string.md)                                              | -                       |
| `identifiers`    | Any kind of identifier for any kind of Thing. | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | -                       |
| `images`         | Images of the item.                           | [`ImageObject`](./image-object.md)*                                  | -                       |
| `name`           | The name of the item.                         | [`String`](./string.md)                                              | -                       |
| `url`            | The URL of the item.                          | [`String`](./string.md)                                              | -                       |
| `id`             | The identifier for this item.                 | [`String`](./string.md)                                              | [`Entity`](./entity.md) |

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
