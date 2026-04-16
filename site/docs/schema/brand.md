---
title: Brand
description: A brand used by an organization or person for labeling a product, product group, or similar.
---

This is an implementation of schema.org [`Brand`](https://schema.org/Brand).

In Stencila Schema it is used as structured metadata on products,
organizations, and related entities while remaining interoperable with the
schema.org vocabulary. It mainly serves bibliographic and descriptive use
cases rather than introducing document-specific behavior of its own.

Key properties are generally inherited from [`Thing`](./thing.md), especially
`name`, `description`, identifiers, and URLs.


# Analogues

The following external types, elements, or nodes are similar to a `Brand`:

- schema.org [`Brand`](https://schema.org/Brand)

# Properties

The `Brand` type has these properties:

| Name             | Description                                   | Type                                                                 | Inherited from          |
| ---------------- | --------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `logo`           | A logo associated with the brand.             | [`ImageObject`](./image-object.md)                                   | -                       |
| `reviews`        | Reviews of the brand.                         | [`String`](./string.md)*                                             | -                       |
| `alternateNames` | Alternate names (aliases) for the item.       | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`    | A description of the item.                    | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`    | Any kind of identifier for any kind of Thing. | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`         | Images of the item.                           | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`           | The name of the item.                         | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`            | The URL of the item.                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `id`             | The identifier for this item.                 | [`String`](./string.md)                                              | [`Entity`](./entity.md) |

# Related

The `Brand` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Bindings

The `Brand` type is represented in:

- [JSON-LD](https://stencila.org/Brand.jsonld)
- [JSON Schema](https://stencila.org/Brand.schema.json)
- Python class [`Brand`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Brand`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/brand.rs)
- TypeScript class [`Brand`](https://github.com/stencila/stencila/blob/main/ts/src/types/Brand.ts)

***

This documentation was generated from [`Brand.yaml`](https://github.com/stencila/stencila/blob/main/schema/Brand.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
