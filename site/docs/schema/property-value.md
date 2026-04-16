---
title: Property Value
description: A property-value pair.
---

This is an implementation of schema.org
[`PropertyValue`](https://schema.org/PropertyValue).

It is used for generic property-value metadata when no more specific schema
property is available. In Stencila Schema it is especially useful for
structured identifiers and similar descriptive metadata attached to
[`Thing`](./thing.md) and derived types.

Key properties include `propertyID` and `value`. Prefer more specific
properties when they exist, because they usually carry stronger semantics.


# Properties

The `PropertyValue` type has these properties:

| Name             | Description                                                                    | Type                                                                 | Inherited from          |
| ---------------- | ------------------------------------------------------------------------------ | -------------------------------------------------------------------- | ----------------------- |
| `propertyID`     | A commonly used identifier for the characteristic represented by the property. | [`String`](./string.md)                                              | -                       |
| `value`          | The value of the property.                                                     | [`Primitive`](./primitive.md)                                        | -                       |
| `alternateNames` | Alternate names (aliases) for the item.                                        | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`    | A description of the item.                                                     | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`    | Any kind of identifier for any kind of Thing.                                  | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`         | Images of the item.                                                            | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`           | The name of the item.                                                          | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`            | The URL of the item.                                                           | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `id`             | The identifier for this item.                                                  | [`String`](./string.md)                                              | [`Entity`](./entity.md) |

# Related

The `PropertyValue` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Bindings

The `PropertyValue` type is represented in:

- [JSON-LD](https://stencila.org/PropertyValue.jsonld)
- [JSON Schema](https://stencila.org/PropertyValue.schema.json)
- Python class [`PropertyValue`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`PropertyValue`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/property_value.rs)
- TypeScript class [`PropertyValue`](https://github.com/stencila/stencila/blob/main/ts/src/types/PropertyValue.ts)

***

This documentation was generated from [`PropertyValue.yaml`](https://github.com/stencila/stencila/blob/main/schema/PropertyValue.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
