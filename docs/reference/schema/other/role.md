---
title: Role
description: Represents additional information about a relationship or property.
config:
  publish:
    ghost:
      type: post
      slug: role
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Other
---

For example a `Role` can be used to say that a 'member' role linking some `SportsTeam` 
to a player occurred during a particular time period. Or that a `Person`'s 'actor' role in a `Movie`
was for some particular `characterName`. Such properties can be attached to a `Role` entity,
which is then associated with the main entities using ordinary properties like 'member' or 'actor'.

See http://blog.schema.org/2014/06/introducing-role.html.


# Properties

The `Role` type has these properties:

| Name | Description                   | Type                                                               | Inherited from                                                     | `JSON-LD @id`                        | Aliases |
| ---- | ----------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------ | ------------------------------------ | ------- |
| `id` | The identifier for this item. | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -       |

# Related

The `Role` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: [`AuthorRole`](https://stencila.ghost.io/docs/reference/schema/author-role)

# Bindings

The `Role` type is represented in:

- [JSON-LD](https://stencila.org/Role.jsonld)
- [JSON Schema](https://stencila.org/Role.schema.json)
- Python class [`Role`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/role.py)
- Rust struct [`Role`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/role.rs)
- TypeScript class [`Role`](https://github.com/stencila/stencila/blob/main/ts/src/types/Role.ts)

# Source

This documentation was generated from [`Role.yaml`](https://github.com/stencila/stencila/blob/main/schema/Role.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
