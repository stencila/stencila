---
title: Product
description: Any offered product or service. For example, a pair of shoes; a haircut; or an episode of a TV show streamed online.
---

The `Product` type allows you to provide details about a product such as the product
brand, logo and ID. This type can be used as any kind of product that is not [`CreativeWork`](./creative-work.md) item.

`Product` is analogous, and structurally similar to, the 
JATS XML [`<product>`](https://jats.nlm.nih.gov/archiving/tag-library/1.1/element/product.html) element which
contains the metadata concerning one product (for example, a book, software package, website, or
hardware component) discussed in an article.

To meet Google's guidelines for [`Product`](https://developers.google.com/search/docs/data-types/product#product)
instances are required to have `image` and `name` properties.


# Properties

The `Product` type has these properties:

| Name             | Description                                   | Type                                                                 | Inherited from          |
| ---------------- | --------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `id`             | The identifier for this item.                 | [`String`](./string.md)                                              | [`Entity`](./entity.md) |
| `alternateNames` | Alternate names (aliases) for the item.       | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`    | A description of the item.                    | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`    | Any kind of identifier for any kind of Thing. | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`         | Images of the item.                           | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`           | The name of the item.                         | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`            | The URL of the item.                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `brands`         | Brands that the product is labelled with.     | [`Brand`](./brand.md)*                                               | -                       |
| `logo`           | The logo of the product.                      | [`ImageObject`](./image-object.md)                                   | -                       |
| `productID`      | Product identification code.                  | [`String`](./string.md)                                              | -                       |

# Related

The `Product` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Bindings

The `Product` type is represented in:

- [JSON-LD](https://stencila.org/Product.jsonld)
- [JSON Schema](https://stencila.org/Product.schema.json)
- Python class [`Product`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/product.py)
- Rust struct [`Product`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/product.rs)
- TypeScript class [`Product`](https://github.com/stencila/stencila/blob/main/ts/src/types/Product.ts)

# Source

This documentation was generated from [`Product.yaml`](https://github.com/stencila/stencila/blob/main/schema/Product.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
