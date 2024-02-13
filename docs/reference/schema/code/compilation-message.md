# Compilation Message

**An error, warning or log message generated during compilation.**

**`@id`**: `stencila:CompilationMessage`

This type is marked as unstable and is subject to change.

## Properties

The `CompilationMessage` type has these properties:

| Name           | Aliases                          | `@id`                                | Type                                                                                                          | Description                                                | Inherited from                                                                                   |
| -------------- | -------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`           | -                                | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)               | The identifier for this item.                              | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `level`        | -                                | `stencila:level`                     | [`MessageLevel`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/message-level.md) | The severity level of the message.                         | -                                                                                                |
| `message`      | -                                | `stencila:message`                   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)               | The text of the message.                                   | -                                                                                                |
| `errorType`    | `error-type`, `error_type`       | `stencila:errorType`                 | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)               | The type of error e.g. "SyntaxError", "ZeroDivisionError". | -                                                                                                |
| `codeLocation` | `code-location`, `code_location` | `stencila:codeLocation`              | [`CodeLocation`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/code-location.md)  | The location that the error occurred.                      | -                                                                                                |

## Related

The `CompilationMessage` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `CompilationMessage` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding     | Decoding  | Status              | Notes |
| -------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.md)           | 🟢 No loss    |           | 🚧 Under development |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss   |           | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |              |           | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)    |              |           | 🚧 Under development |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss   |           | 🟢 Stable            |       |

## Bindings

The `CompilationMessage` type is represented in these bindings:

- [JSON-LD](https://stencila.org/CompilationMessage.jsonld)
- [JSON Schema](https://stencila.org/CompilationMessage.schema.json)
- Python class [`CompilationMessage`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/compilation_message.py)
- Rust struct [`CompilationMessage`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/compilation_message.rs)
- TypeScript class [`CompilationMessage`](https://github.com/stencila/stencila/blob/main/ts/src/types/CompilationMessage.ts)

## Source

This documentation was generated from [`CompilationMessage.yaml`](https://github.com/stencila/stencila/blob/main/schema/CompilationMessage.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).