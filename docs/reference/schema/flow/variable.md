# Variable

**A variable representing a name / value pair.**

**`@id`**: `stencila:Variable`

This type is marked as unstable and is subject to change.

## Properties

The `Variable` type has these properties:

| Name                  | Aliases                                        | `@id`                                                                  | Type                                                                                            | Description                                                                           | Inherited from                                                                                   |
| --------------------- | ---------------------------------------------- | ---------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`                  | -                                              | [`schema:id`](https://schema.org/id)                                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item.                                                         | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `name`                | -                                              | [`schema:name`](https://schema.org/name)                               | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The name of the variable.                                                             | -                                                                                                |
| `programmingLanguage` | `programming-language`, `programming_language` | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The programming language that the variable is defined in e.g. Python, JSON.           | -                                                                                                |
| `nativeType`          | `native-type`, `native_type`                   | `stencila:nativeType`                                                  | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The native type of the variable e.g. `float`, `datetime.datetime`, `pandas.DataFrame` | -                                                                                                |
| `nodeType`            | `node-type`, `node_type`                       | `stencila:nodeType`                                                    | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The Stencila node type of the variable e.g. `Number`, `DateTime`, `Datatable`.        | -                                                                                                |
| `value`               | -                                              | [`schema:value`](https://schema.org/value)                             | [`Node`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/node.md)    | The value of the variable.                                                            | -                                                                                                |
| `hint`                | -                                              | `stencila:hint`                                                        | [`Hint`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/hint.md)     | A hint of the value and/or structure of the variable.                                 | -                                                                                                |

## Related

The `Variable` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Variable` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding  | Status              | Notes |
| ---------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |           | 🚧 Under development |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |           | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |           | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |           | 🚧 Under development |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |           | 🟢 Stable            |       |

## Bindings

The `Variable` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Variable.jsonld)
- [JSON Schema](https://stencila.org/Variable.schema.json)
- Python class [`Variable`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/variable.py)
- Rust struct [`Variable`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/variable.rs)
- TypeScript class [`Variable`](https://github.com/stencila/stencila/blob/main/ts/src/types/Variable.ts)

## Source

This documentation was generated from [`Variable.yaml`](https://github.com/stencila/stencila/blob/main/schema/Variable.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).