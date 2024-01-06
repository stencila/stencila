# Role

**Represents additional information about a relationship or property.**

For example a `Role` can be used to say that a 'member' role linking some `SportsTeam` 
to a player occurred during a particular time period. Or that a `Person`'s 'actor' role in a `Movie`
was for some particular `characterName`. Such properties can be attached to a `Role` entity,
which is then associated with the main entities using ordinary properties like 'member' or 'actor'.

See http://blog.schema.org/2014/06/introducing-role.html.


**`@id`**: [`schema:Role`](https://schema.org/Role)

## Properties

The `Role` type has these properties:

| Name | Aliases | `@id`                                | Type                                                                                            | Description                   | Inherited from                                                                                   |
| ---- | ------- | ------------------------------------ | ----------------------------------------------------------------------------------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------ |
| `id` | -       | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item. | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |

## Related

The `Role` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: [`AuthorRole`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author-role.md)

## Formats

The `Role` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `Role` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Role.jsonld)
- [JSON Schema](https://stencila.org/Role.schema.json)
- Python class [`Role`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/role.py)
- Rust struct [`Role`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/role.rs)
- TypeScript class [`Role`](https://github.com/stencila/stencila/blob/main/ts/src/types/Role.ts)

## Source

This documentation was generated from [`Role.yaml`](https://github.com/stencila/stencila/blob/main/schema/Role.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).