---
title: Author
description: A union type for authors of a `CreativeWork` or other type.
---

This is a union type used in Stencila Schema for authorship values.

It exists to provide a readable alias for the set of node types that can act
as an author in Stencila Schema, instead of repeating or generating the much
longer union name `PersonOrOrganizationOrSoftwareApplicationOrAuthorRole`.

See properties such as [`CreativeWork.authors`](./creative-work.md#authors)
and [`CreativeWork.contributors`](./creative-work.md#contributors) for common
uses of this union.


# Analogues

The following external types, elements, or nodes are similar to a `Author`:

- [schema.org author range](https://schema.org/author): Close analogue for the set of entities that may fill the schema.org `author` property, though Stencila extends this with `SoftwareApplication` and `AuthorRole` in one reusable union.

# Members

The `Author` type has these members:

- [`Person`](./person.md)
- [`Organization`](./organization.md)
- [`SoftwareApplication`](./software-application.md)
- [`AuthorRole`](./author-role.md)

# Bindings

The `Author` type is represented in:

- [JSON-LD](https://stencila.org/Author.jsonld)
- [JSON Schema](https://stencila.org/Author.schema.json)
- Python type [`Author`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`Author`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/author.rs)
- TypeScript type [`Author`](https://github.com/stencila/stencila/blob/main/ts/src/types/Author.ts)

***

This documentation was generated from [`Author.yaml`](https://github.com/stencila/stencila/blob/main/schema/Author.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
