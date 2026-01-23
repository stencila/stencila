---
title: Chat Message Group
description: A group of messages, usually alternative model messages, within a `Chat`.
---

This type is marked as unstable and is subject to change.

# Properties

The `ChatMessageGroup` type has these properties:

| Name       | Description                    | Type                                | Inherited from          |
| ---------- | ------------------------------ | ----------------------------------- | ----------------------- |
| `id`       | The identifier for this item.  | [`String`](./string.md)             | [`Entity`](./entity.md) |
| `messages` | The messages within the group. | [`ChatMessage`](./chat-message.md)* | -                       |

# Related

The `ChatMessageGroup` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ChatMessageGroup` type is represented in:

- [JSON-LD](https://stencila.org/ChatMessageGroup.jsonld)
- [JSON Schema](https://stencila.org/ChatMessageGroup.schema.json)
- Python class [`ChatMessageGroup`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ChatMessageGroup`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/chat_message_group.rs)
- TypeScript class [`ChatMessageGroup`](https://github.com/stencila/stencila/blob/main/ts/src/types/ChatMessageGroup.ts)

***

This documentation was generated from [`ChatMessageGroup.yaml`](https://github.com/stencila/stencila/blob/main/schema/ChatMessageGroup.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
