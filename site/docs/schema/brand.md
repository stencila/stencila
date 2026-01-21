---
title: Brand
description: A brand used by an organization or person for labeling a product, product group, or similar.
---

# Properties

The `Brand` type has these properties:

| Name             | Description                                   | Type                                                                 | Inherited from          |
| ---------------- | --------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `id`             | The identifier for this item.                 | [`String`](./string.md)                                              | [`Entity`](./entity.md) |
| `alternateNames` | Alternate names (aliases) for the item.       | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`    | A description of the item.                    | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`    | Any kind of identifier for any kind of Thing. | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`         | Images of the item.                           | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`           | The name of the item.                         | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`            | The URL of the item.                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `logo`           | A logo associated with the brand.             | [`ImageObject`](./image-object.md)                                   | -                       |
| `reviews`        | Reviews of the brand.                         | [`String`](./string.md)*                                             | -                       |

# Related

The `Brand` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Bindings

The `Brand` type is represented in:

- [JSON-LD](https://stencila.org/Brand.jsonld)
- [JSON Schema](https://stencila.org/Brand.schema.json)
- Python class [`Brand`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/brand.py)
- Rust struct [`Brand`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/brand.rs)
- TypeScript class [`Brand`](https://github.com/stencila/stencila/blob/main/ts/src/types/Brand.ts)

# Source

This documentation was generated from [`Brand.yaml`](https://github.com/stencila/stencila/blob/main/schema/Brand.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
