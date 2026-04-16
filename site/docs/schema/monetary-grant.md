---
title: Monetary Grant
description: A monetary grant.
---

This is an implementation of schema.org
[`MonetaryGrant`](https://schema.org/MonetaryGrant).

It specializes [`Grant`](./grant.md) for funding that has an explicit monetary
value, while integrating with Stencila's structured metadata model for works
and contributors.

Key properties include inherited grant metadata together with the monetary
amount and related funding information.


# Analogues

The following external types, elements, or nodes are similar to a `MonetaryGrant`:

- schema.org [`MonetaryGrant`](https://schema.org/MonetaryGrant)

# Properties

The `MonetaryGrant` type has these properties:

| Name             | Description                                                                                              | Type                                                                 | Inherited from          |
| ---------------- | -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `amounts`        | The amount of money.                                                                                     | [`Number`](./number.md)                                              | -                       |
| `funders`        | A person or organization that supports (sponsors) something through some kind of financial contribution. | ([`Person`](./person.md) \| [`Organization`](./organization.md))*    | -                       |
| `fundedItems`    | Indicates an item funded or sponsored through a Grant.                                                   | [`ThingVariant`](./thing-variant.md)*                                | [`Grant`](./grant.md)   |
| `sponsors`       | A person or organization that supports a thing through a pledge, promise, or financial contribution.     | ([`Person`](./person.md) \| [`Organization`](./organization.md))*    | [`Grant`](./grant.md)   |
| `alternateNames` | Alternate names (aliases) for the item.                                                                  | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`    | A description of the item.                                                                               | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`    | Any kind of identifier for any kind of Thing.                                                            | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`         | Images of the item.                                                                                      | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`           | The name of the item.                                                                                    | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`            | The URL of the item.                                                                                     | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `id`             | The identifier for this item.                                                                            | [`String`](./string.md)                                              | [`Entity`](./entity.md) |

# Related

The `MonetaryGrant` type is related to these types:

- Parents: [`Grant`](./grant.md)
- Children: none

# Bindings

The `MonetaryGrant` type is represented in:

- [JSON-LD](https://stencila.org/MonetaryGrant.jsonld)
- [JSON Schema](https://stencila.org/MonetaryGrant.schema.json)
- Python class [`MonetaryGrant`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`MonetaryGrant`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/monetary_grant.rs)
- TypeScript class [`MonetaryGrant`](https://github.com/stencila/stencila/blob/main/ts/src/types/MonetaryGrant.ts)

***

This documentation was generated from [`MonetaryGrant.yaml`](https://github.com/stencila/stencila/blob/main/schema/MonetaryGrant.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
