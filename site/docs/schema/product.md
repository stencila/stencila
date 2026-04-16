---
title: Product
description: A product or service.
---

This is an implementation of schema.org [`Product`](https://schema.org/Product).

In Stencila Schema it is used for products and services that are discussed,
cited, or described in documents, while remaining interoperable with
schema.org metadata conventions.

Key properties include `brands`, `logo`, `productID`, and inherited metadata
from [`Thing`](./thing.md).


# Analogues

The following external types, elements, or nodes are similar to a `Product`:

- schema.org [`Product`](https://schema.org/Product)
- JATS [`<product>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/product.html)

# Properties

The `Product` type has these properties:

| Name             | Description                                   | Type                                                                 | Inherited from          |
| ---------------- | --------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `brands`         | Brands that the product is labelled with.     | [`Brand`](./brand.md)*                                               | -                       |
| `logo`           | The logo of the product.                      | [`ImageObject`](./image-object.md)                                   | -                       |
| `productID`      | Product identification code.                  | [`String`](./string.md)                                              | -                       |
| `alternateNames` | Alternate names (aliases) for the item.       | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`    | A description of the item.                    | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`    | Any kind of identifier for any kind of Thing. | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`         | Images of the item.                           | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`           | The name of the item.                         | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`            | The URL of the item.                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `id`             | The identifier for this item.                 | [`String`](./string.md)                                              | [`Entity`](./entity.md) |

# Related

The `Product` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Bindings

The `Product` type is represented in:

- [JSON-LD](https://stencila.org/Product.jsonld)
- [JSON Schema](https://stencila.org/Product.schema.json)
- Python class [`Product`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Product`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/product.rs)
- TypeScript class [`Product`](https://github.com/stencila/stencila/blob/main/ts/src/types/Product.ts)

***

This documentation was generated from [`Product.yaml`](https://github.com/stencila/stencila/blob/main/schema/Product.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
