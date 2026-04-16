---
title: Message Level
description: A severity level for a message.
---

This is an enumeration used in Stencila Schema for diagnostic severity levels.

It exists so compilation, execution, and other message-producing workflows can
communicate severity in a consistent way across tools and renderers.

See [`CompilationMessage`](./compilation-message.md) and
[`ExecutionMessage`](./execution-message.md) for the main types that use this
enumeration.


# Members

The `MessageLevel` type has these members:

| Member      | Description            |
| ----------- | ---------------------- |
| `Trace`     | A tracing message      |
| `Debug`     | A debug message        |
| `Info`      | An information message |
| `Warning`   | A warning message      |
| `Error`     | An error message       |
| `Exception` | An exception message   |

# Bindings

The `MessageLevel` type is represented in:

- [JSON-LD](https://stencila.org/MessageLevel.jsonld)
- [JSON Schema](https://stencila.org/MessageLevel.schema.json)
- Python type [`MessageLevel`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`MessageLevel`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/message_level.rs)
- TypeScript type [`MessageLevel`](https://github.com/stencila/stencila/blob/main/ts/src/types/MessageLevel.ts)

***

This documentation was generated from [`MessageLevel.yaml`](https://github.com/stencila/stencila/blob/main/schema/MessageLevel.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
