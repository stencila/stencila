# Variable

**A variable representing a name / value pair.**

**`@id`**: `stencila:Variable`

This type is marked as experimental and is likely to change.

## Properties

The `Variable` type has these properties:

| Name      | `@id`                                      | Type                                                                                            | Description                                                               | Inherited from                                                                                      |
| --------- | ------------------------------------------ | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| id        | [`schema:id`](https://schema.org/id)       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item                                              | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)    |
| namespace | `stencila:namespace`                       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The namespace, usually a document path, within which the variable resides | [`Variable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/variable.md) |
| name      | [`schema:name`](https://schema.org/name)   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The name of the variable.                                                 | [`Variable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/variable.md) |
| kind      | `stencila:kind`                            | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The expected type of variable e.g. `Number`, `Timestamp`, `Datatable`     | [`Variable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/variable.md) |
| value     | [`schema:value`](https://schema.org/value) | [`Node`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/node.md)    | The value of the variable.                                                | [`Variable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/variable.md) |

## Related

The `Variable` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Variable` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                            | Encoding       | Decoding     | Status                 | Notes |
| ------------------------------------------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/HTML.md)             | 🔷 Low loss     |              | 🚧 Under development    |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JATS.md)             | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Markdown.md)     | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Plain text.md) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JSON.md)             | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/JSON5.md)           | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/YAML.md)             | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/Debug.md)           | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `Variable` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Variable.jsonld)
- [JSON Schema](https://stencila.dev/Variable.schema.json)
- Python class [`Variable`](https://github.com/stencila/stencila/blob/main/python/stencila/types/variable.py)
- Rust struct [`Variable`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/variable.rs)
- TypeScript class [`Variable`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Variable.ts)

## Source

This documentation was generated from [`Variable.yaml`](https://github.com/stencila/stencila/blob/main/schema/Variable.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).