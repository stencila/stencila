---
title: Author Role Author
description: A union type for authors in an `AuthorRole`.
---

This is a union type used in Stencila Schema for the `author` property of
[`AuthorRole`](./author-role.md).

It exists to provide a readable alias for the node types that can appear in
that property, instead of using the much longer generated union name
`PersonOrOrganizationOrSoftwareApplicationOrThing`. The `Thing` variant is
intended only for anonymous or otherwise unidentified authors when none of the
more specific variants is appropriate.

See [`AuthorRole.author`](./author-role.md#author) for the property that uses
this union.


# Analogues

The following external types, elements, or nodes are similar to a `AuthorRoleAuthor`:

- [schema.org Role range pattern](https://schema.org/Role): Approximate analogue for entities that can fill a role-bearing property, with Stencila adding an explicit fallback to `Thing` for anonymous or unidentified authors.

# Members

The `AuthorRoleAuthor` type has these members:

- [`Person`](./person.md)
- [`Organization`](./organization.md)
- [`SoftwareApplication`](./software-application.md)
- [`Thing`](./thing.md)

# Bindings

The `AuthorRoleAuthor` type is represented in:

- [JSON-LD](https://stencila.org/AuthorRoleAuthor.jsonld)
- [JSON Schema](https://stencila.org/AuthorRoleAuthor.schema.json)
- Python type [`AuthorRoleAuthor`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`AuthorRoleAuthor`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/author_role_author.rs)
- TypeScript type [`AuthorRoleAuthor`](https://github.com/stencila/stencila/blob/main/ts/src/types/AuthorRoleAuthor.ts)

***

This documentation was generated from [`AuthorRoleAuthor.yaml`](https://github.com/stencila/stencila/blob/main/schema/AuthorRoleAuthor.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
