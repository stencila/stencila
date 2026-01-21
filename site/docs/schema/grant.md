---
title: Grant
description: A grant, typically financial or otherwise quantifiable, of resources.
---

# Properties

The `Grant` type has these properties:

| Name             | Description                                                                                          | Type                                                                 | Inherited from          |
| ---------------- | ---------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `id`             | The identifier for this item.                                                                        | [`String`](./string.md)                                              | [`Entity`](./entity.md) |
| `alternateNames` | Alternate names (aliases) for the item.                                                              | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`    | A description of the item.                                                                           | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`    | Any kind of identifier for any kind of Thing.                                                        | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`         | Images of the item.                                                                                  | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`           | The name of the item.                                                                                | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`            | The URL of the item.                                                                                 | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `fundedItems`    | Indicates an item funded or sponsored through a Grant.                                               | [`ThingVariant`](./thing-variant.md)*                                | -                       |
| `sponsors`       | A person or organization that supports a thing through a pledge, promise, or financial contribution. | ([`Person`](./person.md) \| [`Organization`](./organization.md))*    | -                       |

# Related

The `Grant` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: [`MonetaryGrant`](./monetary-grant.md)

# Bindings

The `Grant` type is represented in:

- [JSON-LD](https://stencila.org/Grant.jsonld)
- [JSON Schema](https://stencila.org/Grant.schema.json)
- Python class [`Grant`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/grant.py)
- Rust struct [`Grant`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/grant.rs)
- TypeScript class [`Grant`](https://github.com/stencila/stencila/blob/main/ts/src/types/Grant.ts)

# Source

This documentation was generated from [`Grant.yaml`](https://github.com/stencila/stencila/blob/main/schema/Grant.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
