---
title: Author Role Author
description: Union type for things that can be an author in `AuthorRole`.
---

This type merely exists to avoid the excessively long type name
(`PersonOrOrganizationOrSoftwareApplicationOrThing`) that is otherwise generated.
The `Thing` variant is intended only for anonymous authors which is not known to
be one the other variants and which should be given the name "anon".


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
- Python type [`AuthorRoleAuthor`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/author_role_author.py)
- Rust type [`AuthorRoleAuthor`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/author_role_author.rs)
- TypeScript type [`AuthorRoleAuthor`](https://github.com/stencila/stencila/blob/main/ts/src/types/AuthorRoleAuthor.ts)

# Source

This documentation was generated from [`AuthorRoleAuthor.yaml`](https://github.com/stencila/stencila/blob/main/schema/AuthorRoleAuthor.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
