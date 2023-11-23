# Compilation Error

**An error that occurred while compiling an executable node.**

**`@id`**: `stencila:CompilationError`

This type is marked as unstable and is subject to change.

## Properties

The `CompilationError` type has these properties:

| Name           | Aliases                                     | `@id`                                | Type                                                                                                         | Description                                                | Inherited from                                                                                   |
| -------------- | ------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`           | -                                           | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)              | The identifier for this item.                              | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `errorMessage` | `message`, `error-message`, `error_message` | `stencila:errorMessage`              | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)              | The error message or brief description of the error.       | -                                                                                                |
| `errorType`    | `error-type`, `error_type`                  | `stencila:errorType`                 | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)              | The type of error e.g. "SyntaxError", "ZeroDivisionError". | -                                                                                                |
| `codeLocation` | `code-location`, `code_location`            | `stencila:codeLocation`              | [`CodeLocation`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/code-location.md) | The location that the error occurred.                      | -                                                                                                |

## Related

The `CompilationError` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `CompilationError` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `CompilationError` type is represented in these bindings:

- [JSON-LD](https://stencila.org/CompilationError.jsonld)
- [JSON Schema](https://stencila.org/CompilationError.schema.json)
- Python class [`CompilationError`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/compilation_error.py)
- Rust struct [`CompilationError`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/compilation_error.rs)
- TypeScript class [`CompilationError`](https://github.com/stencila/stencila/blob/main/typescript/src/types/CompilationError.ts)

## Source

This documentation was generated from [`CompilationError.yaml`](https://github.com/stencila/stencila/blob/main/schema/CompilationError.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).