---
title: Author
description: Union type for things that can be an author of a `CreativeWork` or other type.
---

This type merely exists to avoid the excessively long type name
(`PersonOrOrganizationOrSoftwareApplicationOrAuthorRole`) that is otherwise generated.


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
- Python type [`Author`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/author.py)
- Rust type [`Author`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/author.rs)
- TypeScript type [`Author`](https://github.com/stencila/stencila/blob/main/ts/src/types/Author.ts)

# Source

This documentation was generated from [`Author.yaml`](https://github.com/stencila/stencila/blob/main/schema/Author.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
