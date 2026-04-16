---
title: Author Role
description: An author and their role.
---

This is a Stencila Schema specialization of schema.org
[`Role`](https://schema.org/Role) for authorship.

It exists to represent not just who contributed to a work, but how they
contributed and in what format or editing context. This supports richer
provenance, attribution, and contribution tracking than a flat list of authors
alone.

Key properties include `author`, `roleName`, `format`, and `lastModified`.


# Analogues

The following external types, elements, or nodes are similar to a `AuthorRole`:

- schema.org [`Role`](https://schema.org/Role): Direct schema.org source pattern, specialized by Stencila for authorship and contribution provenance.
- [CRediT contributor role assertion](https://credit.niso.org/): Approximate analogue for attributed contributor roles, though Stencila additionally records author identity, format, and modification metadata in one node.

# Properties

The `AuthorRole` type has these properties:

| Name           | Description                                                                | Type                                          | Inherited from          |
| -------------- | -------------------------------------------------------------------------- | --------------------------------------------- | ----------------------- |
| `author`       | The entity acting as an author.                                            | [`AuthorRoleAuthor`](./author-role-author.md) | -                       |
| `roleName`     | The role played by the author.                                             | [`AuthorRoleName`](./author-role-name.md)     | -                       |
| `format`       | The format that the author used to perform the role. e.g. Markdown, Python | [`String`](./string.md)                       | -                       |
| `lastModified` | Timestamp of most recent modification, by the author, in the role.         | [`Timestamp`](./timestamp.md)                 | -                       |
| `id`           | The identifier for this item.                                              | [`String`](./string.md)                       | [`Entity`](./entity.md) |

# Related

The `AuthorRole` type is related to these types:

- Parents: [`Role`](./role.md)
- Children: none

# Bindings

The `AuthorRole` type is represented in:

- [JSON-LD](https://stencila.org/AuthorRole.jsonld)
- [JSON Schema](https://stencila.org/AuthorRole.schema.json)
- Python class [`AuthorRole`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`AuthorRole`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/author_role.rs)
- TypeScript class [`AuthorRole`](https://github.com/stencila/stencila/blob/main/ts/src/types/AuthorRole.ts)

***

This documentation was generated from [`AuthorRole.yaml`](https://github.com/stencila/stencila/blob/main/schema/AuthorRole.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
