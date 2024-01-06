# Author

**Union type for things that can be an author of a `CreativeWork` or other type.**

This type merely exists to avoid the excessively long type name
(`PersonOrOrganizationOrSoftwareApplicationOrAuthorRole`) that is otherwise generated.


**`@id`**: `stencila:Author`

## Members

The `Author` type has these members:

- [`Person`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/person.md)
- [`Organization`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/organization.md)
- [`SoftwareApplication`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/software-application.md)
- [`AuthorRole`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author-role.md)

## Bindings

The `Author` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Author.jsonld)
- [JSON Schema](https://stencila.org/Author.schema.json)
- Python type [`Author`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/author.py)
- Rust type [`Author`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/author.rs)
- TypeScript type [`Author`](https://github.com/stencila/stencila/blob/main/ts/src/types/Author.ts)

## Source

This documentation was generated from [`Author.yaml`](https://github.com/stencila/stencila/blob/main/schema/Author.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).