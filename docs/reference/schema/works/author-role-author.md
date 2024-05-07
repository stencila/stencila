# Author Role Author

**Union type for things that can be an author in `AuthorRole`.**

This type merely exists to avoid the excessively long type name
(`PersonOrOrganizationOrSoftwareApplicationOrThing`) that is otherwise generated.
The `Thing` variant is intended only for anonymous authors which is not known to
be one the other variants and which should be given the name "anon".


**`@id`**: `stencila:AuthorRoleAuthor`

## Members

The `AuthorRoleAuthor` type has these members:

- [`Person`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/person.md)
- [`Organization`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/organization.md)
- [`SoftwareApplication`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/software-application.md)
- [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)

## Bindings

The `AuthorRoleAuthor` type is represented in these bindings:

- [JSON-LD](https://stencila.org/AuthorRoleAuthor.jsonld)
- [JSON Schema](https://stencila.org/AuthorRoleAuthor.schema.json)
- Python type [`AuthorRoleAuthor`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/author_role_author.py)
- Rust type [`AuthorRoleAuthor`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/author_role_author.rs)
- TypeScript type [`AuthorRoleAuthor`](https://github.com/stencila/stencila/blob/main/ts/src/types/AuthorRoleAuthor.ts)

## Source

This documentation was generated from [`AuthorRoleAuthor.yaml`](https://github.com/stencila/stencila/blob/main/schema/AuthorRoleAuthor.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
