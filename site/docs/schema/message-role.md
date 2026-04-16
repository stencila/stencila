---
title: Message Role
description: A role in a message exchange.
---

This is an enumeration used in Stencila Schema for message roles in conversational or
instruction contexts.

It exists so user, assistant, system, and related message roles can be
represented consistently across chats, prompts, and model interactions.

See [`ChatMessage`](./chat-message.md) and related messaging types for where
this enumeration is used.


# Members

The `MessageRole` type has these members:

| Member   | Description            |
| -------- | ---------------------- |
| `System` | A system message       |
| `User`   | A user message         |
| `Model`  | A message from a model |

# Bindings

The `MessageRole` type is represented in:

- [JSON-LD](https://stencila.org/MessageRole.jsonld)
- [JSON Schema](https://stencila.org/MessageRole.schema.json)
- Python type [`MessageRole`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`MessageRole`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/message_role.rs)
- TypeScript type [`MessageRole`](https://github.com/stencila/stencila/blob/main/ts/src/types/MessageRole.ts)

***

This documentation was generated from [`MessageRole.yaml`](https://github.com/stencila/stencila/blob/main/schema/MessageRole.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
