---
title: Property Value
description: A property-value pair.
---

Always use specific properties when they exist and you can populate them.
Using `PropertyValue` as a substitute will typically not convey as much semantic
information as using the specific property.

Most of the time a `PropertyValue` node will need a `value` property
(e.g. most of the examples on https://schema.org/PropertyValue have one)
so this schema make that property required.

This type is mainly provided for use in `Thing.identifiers` (see the notes there).


# Properties

The `PropertyValue` type has these properties:

| Name             | Description                                                                    | Type                                                                 | Inherited from          |
| ---------------- | ------------------------------------------------------------------------------ | -------------------------------------------------------------------- | ----------------------- |
| `id`             | The identifier for this item.                                                  | [`String`](./string.md)                                              | [`Entity`](./entity.md) |
| `alternateNames` | Alternate names (aliases) for the item.                                        | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`    | A description of the item.                                                     | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`    | Any kind of identifier for any kind of Thing.                                  | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`         | Images of the item.                                                            | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`           | The name of the item.                                                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`            | The URL of the item.                                                           | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `propertyID`     | A commonly used identifier for the characteristic represented by the property. | [`String`](./string.md)                                              | -                       |
| `value`          | The value of the property.                                                     | [`Primitive`](./primitive.md)                                        | -                       |

# Related

The `PropertyValue` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Bindings

The `PropertyValue` type is represented in:

- [JSON-LD](https://stencila.org/PropertyValue.jsonld)
- [JSON Schema](https://stencila.org/PropertyValue.schema.json)
- Python class [`PropertyValue`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/property_value.py)
- Rust struct [`PropertyValue`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/property_value.rs)
- TypeScript class [`PropertyValue`](https://github.com/stencila/stencila/blob/main/ts/src/types/PropertyValue.ts)

# Source

This documentation was generated from [`PropertyValue.yaml`](https://github.com/stencila/stencila/blob/main/schema/PropertyValue.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
