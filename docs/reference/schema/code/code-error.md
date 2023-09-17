---
title:
- type: Text
  value: CodeError
---

# Code Error

**An error that occurred when parsing, compiling or executing a Code node.**

**`@id`**: `stencila:CodeError`

This type is marked as unstable and is subject to change.

## Properties

The `CodeError` type has these properties:

| Name         | `@id`                                | Type                                                               | Description                                                | Inherited from                                                            |
| ------------ | ------------------------------------ | ------------------------------------------------------------------ | ---------------------------------------------------------- | ------------------------------------------------------------------------- |
| id           | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item                               | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)       |
| errorMessage | `stencila:errorMessage`              | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The error message or brief description of the error.       | [`CodeError`](https://stencila.dev/docs/reference/schema/code/code-error) |
| errorType    | `stencila:errorType`                 | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The type of error e.g. "SyntaxError", "ZeroDivisionError". | [`CodeError`](https://stencila.dev/docs/reference/schema/code/code-error) |
| stackTrace   | `stencila:stackTrace`                | [`String`](https://stencila.dev/docs/reference/schema/data/string) | Stack trace leading up to the error.                       | [`CodeError`](https://stencila.dev/docs/reference/schema/code/code-error) |

## Related

The `CodeError` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `CodeError` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `CodeError` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/CodeError.jsonld)
- [JSON Schema](https://stencila.dev/CodeError.schema.json)
- Python class [`CodeError`](https://github.com/stencila/stencila/blob/main/python/stencila/types/code_error.py)
- Rust struct [`CodeError`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_error.rs)
- TypeScript class [`CodeError`](https://github.com/stencila/stencila/blob/main/typescript/src/types/CodeError.ts)

## Source

This documentation was generated from [`CodeError.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeError.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).