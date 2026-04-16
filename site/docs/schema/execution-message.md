---
title: Execution Message
description: An error, warning or log message generated during execution.
---

This is a type used in Stencila Schema for structured messages produced during
execution.

It exists to capture runtime diagnostics, logs, and warnings as durable
document data instead of transient console output. This lets Stencila render,
serialize, and inspect execution results across environments and sessions.

Key properties typically include message level, text, and optional code or
source-location metadata.


# Properties

The `ExecutionMessage` type has these properties:

| Name           | Description                                                          | Type                                 | Inherited from          |
| -------------- | -------------------------------------------------------------------- | ------------------------------------ | ----------------------- |
| `level`        | The severity level of the message.                                   | [`MessageLevel`](./message-level.md) | -                       |
| `message`      | The text of the message.                                             | [`String`](./string.md)              | -                       |
| `errorType`    | The type of error e.g. "SyntaxError", "ZeroDivisionError".           | [`String`](./string.md)              | -                       |
| `codeLocation` | The location that the error occurred or other message emanated from. | [`CodeLocation`](./code-location.md) | -                       |
| `stackTrace`   | Stack trace leading up to the error.                                 | [`String`](./string.md)              | -                       |
| `id`           | The identifier for this item.                                        | [`String`](./string.md)              | [`Entity`](./entity.md) |

# Related

The `ExecutionMessage` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ExecutionMessage` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionMessage.jsonld)
- [JSON Schema](https://stencila.org/ExecutionMessage.schema.json)
- Python class [`ExecutionMessage`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ExecutionMessage`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_message.rs)
- TypeScript class [`ExecutionMessage`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionMessage.ts)

***

This documentation was generated from [`ExecutionMessage.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionMessage.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
