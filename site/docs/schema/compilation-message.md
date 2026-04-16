---
title: Compilation Message
description: An error, warning or log message generated during compilation.
---

This is a type used in Stencila Schema for messages produced during compilation.

It exists to capture structured diagnostics from compilation-like steps such
as parsing, transpilation, prompt rendering, or overlay expansion. By storing
messages as nodes, Stencila can preserve them across tools, renderers, and
execution sessions.

Key properties typically include message level, text, and optional
source-location information.


# Properties

The `CompilationMessage` type has these properties:

| Name           | Description                                                | Type                                 | Inherited from          |
| -------------- | ---------------------------------------------------------- | ------------------------------------ | ----------------------- |
| `level`        | The severity level of the message.                         | [`MessageLevel`](./message-level.md) | -                       |
| `message`      | The text of the message.                                   | [`String`](./string.md)              | -                       |
| `errorType`    | The type of error e.g. "SyntaxError", "ZeroDivisionError". | [`String`](./string.md)              | -                       |
| `codeLocation` | The location that the error occurred.                      | [`CodeLocation`](./code-location.md) | -                       |
| `id`           | The identifier for this item.                              | [`String`](./string.md)              | [`Entity`](./entity.md) |

# Related

The `CompilationMessage` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `CompilationMessage` type is represented in:

- [JSON-LD](https://stencila.org/CompilationMessage.jsonld)
- [JSON Schema](https://stencila.org/CompilationMessage.schema.json)
- Python class [`CompilationMessage`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`CompilationMessage`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/compilation_message.rs)
- TypeScript class [`CompilationMessage`](https://github.com/stencila/stencila/blob/main/ts/src/types/CompilationMessage.ts)

***

This documentation was generated from [`CompilationMessage.yaml`](https://github.com/stencila/stencila/blob/main/schema/CompilationMessage.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
