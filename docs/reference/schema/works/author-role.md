# Author Role

**An author and their role.**

**`@id`**: `stencila:AuthorRole`

## Properties

The `AuthorRole` type has these properties:

| Name       | Aliases                  | `@id`                                            | Type                                                                                                                                                                                                                                                                                                                                          | Description                   | Inherited from                                                                                   |
| ---------- | ------------------------ | ------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`       | -                        | [`schema:id`](https://schema.org/id)             | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                                                                                                                                               | The identifier for this item. | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `author`   | -                        | [`schema:author`](https://schema.org/author)     | [`Person`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/person.md) \| [`Organization`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/organization.md) \| [`SoftwareApplication`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/software-application.md) | The author.                   | -                                                                                                |
| `roleName` | `role-name`, `role_name` | [`schema:roleName`](https://schema.org/roleName) | [`AuthorRoleName`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author-role-name.md)                                                                                                                                                                                                                            | A role played by the author.  | -                                                                                                |

## Related

The `AuthorRole` type is related to these types:

- Parents: [`Role`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/role.md)
- Children: none

## Formats

The `AuthorRole` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding         | Decoding     | Status                 | Notes |
| -------------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss       |              | 🚧 Under development    |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |                  |              | 🚧 Under development    |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss     |              | ⚠️ Alpha               |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss     |              | ⚠️ Alpha               |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss        | 🟢 No loss    | 🔶 Beta                 |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss       |              | 🟢 Stable               |       |

## Bindings

The `AuthorRole` type is represented in these bindings:

- [JSON-LD](https://stencila.org/AuthorRole.jsonld)
- [JSON Schema](https://stencila.org/AuthorRole.schema.json)
- Python class [`AuthorRole`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/author_role.py)
- Rust struct [`AuthorRole`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/author_role.rs)
- TypeScript class [`AuthorRole`](https://github.com/stencila/stencila/blob/main/ts/src/types/AuthorRole.ts)

## Source

This documentation was generated from [`AuthorRole.yaml`](https://github.com/stencila/stencila/blob/main/schema/AuthorRole.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).