---
title: Role
description: Represents additional information about a relationship or property.
---

This is an implementation of schema.org [`Role`](https://schema.org/Role).

It provides a way to attach additional metadata to a relationship between two
entities, such as a contribution role, time span, or contextual label. In
Stencila Schema it mainly serves as a base for more specific role types such
as [`AuthorRole`](./author-role.md).

See the derived role types for the main Stencila Schema uses of this model.


# Properties

The `Role` type has these properties:

| Name | Description                   | Type                    | Inherited from          |
| ---- | ----------------------------- | ----------------------- | ----------------------- |
| `id` | The identifier for this item. | [`String`](./string.md) | [`Entity`](./entity.md) |

# Related

The `Role` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: [`AuthorRole`](./author-role.md)

# Bindings

The `Role` type is represented in:

- [JSON-LD](https://stencila.org/Role.jsonld)
- [JSON Schema](https://stencila.org/Role.schema.json)
- Python class [`Role`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Role`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/role.rs)
- TypeScript class [`Role`](https://github.com/stencila/stencila/blob/main/ts/src/types/Role.ts)

***

This documentation was generated from [`Role.yaml`](https://github.com/stencila/stencila/blob/main/schema/Role.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
