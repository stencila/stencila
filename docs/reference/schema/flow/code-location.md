# Code Location

**The location within some source code.**

**`@id`**: `stencila:CodeLocation`

## Properties

The `CodeLocation` type has these properties:

| Name          | Aliases                        | `@id`                                | Type                                                                                                               | Description                                                        | Inherited from                                                                                   |
| ------------- | ------------------------------ | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| `id`          | -                              | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The identifier for this item.                                      | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `source`      | -                              | `stencila:source`                    | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                    | The source of the code, a file path, label or URL.                 | -                                                                                                |
| `startLine`   | `start-line`, `start_line`     | `stencila:startLine`                 | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | The 1-based index if the first line on which the error occurred.   | -                                                                                                |
| `startColumn` | `start-column`, `start_column` | `stencila:startColumn`               | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | The 1-based index if the first column on which the error occurred. | -                                                                                                |
| `endLine`     | `end-line`, `end_line`         | `stencila:endLine`                   | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | The 1-based index if the last line on which the error occurred.    | -                                                                                                |
| `endColumn`   | `end-column`, `end_column`     | `stencila:endColumn`                 | [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md) | The 1-based index if the last column on which the error occurred.  | -                                                                                                |

## Related

The `CodeLocation` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `CodeLocation` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `CodeLocation` type is represented in these bindings:

- [JSON-LD](https://stencila.org/CodeLocation.jsonld)
- [JSON Schema](https://stencila.org/CodeLocation.schema.json)
- Python class [`CodeLocation`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/code_location.py)
- Rust struct [`CodeLocation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_location.rs)
- TypeScript class [`CodeLocation`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeLocation.ts)

## Source

This documentation was generated from [`CodeLocation.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeLocation.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).