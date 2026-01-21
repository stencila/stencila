---
title: Author Role
description: An author and their role.
---

# Properties

The `AuthorRole` type has these properties:

| Name           | Description                                                                | Type                                          | Inherited from          |
| -------------- | -------------------------------------------------------------------------- | --------------------------------------------- | ----------------------- |
| `id`           | The identifier for this item.                                              | [`String`](./string.md)                       | [`Entity`](./entity.md) |
| `author`       | The entity acting as an author.                                            | [`AuthorRoleAuthor`](./author-role-author.md) | -                       |
| `roleName`     | The role played by the author.                                             | [`AuthorRoleName`](./author-role-name.md)     | -                       |
| `format`       | The format that the author used to perform the role. e.g. Markdown, Python | [`String`](./string.md)                       | -                       |
| `lastModified` | Timestamp of most recent modification, by the author, in the role.         | [`Timestamp`](./timestamp.md)                 | -                       |

# Related

The `AuthorRole` type is related to these types:

- Parents: [`Role`](./role.md)
- Children: none

# Bindings

The `AuthorRole` type is represented in:

- [JSON-LD](https://stencila.org/AuthorRole.jsonld)
- [JSON Schema](https://stencila.org/AuthorRole.schema.json)
- Python class [`AuthorRole`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/author_role.py)
- Rust struct [`AuthorRole`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/author_role.rs)
- TypeScript class [`AuthorRole`](https://github.com/stencila/stencila/blob/main/ts/src/types/AuthorRole.ts)

# Source

This documentation was generated from [`AuthorRole.yaml`](https://github.com/stencila/stencila/blob/main/schema/AuthorRole.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
