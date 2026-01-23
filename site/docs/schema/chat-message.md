---
title: Chat Message
description: A message within a `Chat`.
---

This type is marked as unstable and is subject to change.

# Properties

The `ChatMessage` type has these properties:

| Name                    | Description                                                                   | Type                                                | Inherited from                  |
| ----------------------- | ----------------------------------------------------------------------------- | --------------------------------------------------- | ------------------------------- |
| `id`                    | The identifier for this item.                                                 | [`String`](./string.md)                             | [`Entity`](./entity.md)         |
| `executionMode`         | Under which circumstances the node should be executed.                        | [`ExecutionMode`](./execution-mode.md)              | [`Executable`](./executable.md) |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.              | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) |
| `compilationMessages`   | Messages generated while compiling the code.                                  | [`CompilationMessage`](./compilation-message.md)*   | [`Executable`](./executable.md) |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.                | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) |
| `executionDependencies` | The upstream dependencies of this node.                                       | [`ExecutionDependency`](./execution-dependency.md)* | [`Executable`](./executable.md) |
| `executionDependants`   | The downstream dependants of this node.                                       | [`ExecutionDependant`](./execution-dependant.md)*   | [`Executable`](./executable.md) |
| `executionTags`         | Tags in the code which affect its execution.                                  | [`ExecutionTag`](./execution-tag.md)*               | [`Executable`](./executable.md) |
| `executionCount`        | A count of the number of times that the node has been executed.               | [`Integer`](./integer.md)                           | [`Executable`](./executable.md) |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.                | [`ExecutionRequired`](./execution-required.md)      | [`Executable`](./executable.md) |
| `executionStatus`       | Status of the most recent, including any current, execution.                  | [`ExecutionStatus`](./execution-status.md)          | [`Executable`](./executable.md) |
| `executionInstance`     | The id of the kernel instance that performed the last execution.              | [`String`](./string.md)                             | [`Executable`](./executable.md) |
| `executionEnded`        | The timestamp when the last execution ended.                                  | [`Timestamp`](./timestamp.md)                       | [`Executable`](./executable.md) |
| `executionDuration`     | Duration of the last execution.                                               | [`Duration`](./duration.md)                         | [`Executable`](./executable.md) |
| `executionMessages`     | Messages emitted while executing the node.                                    | [`ExecutionMessage`](./execution-message.md)*       | [`Executable`](./executable.md) |
| `author`                | The author of the message                                                     | [`Author`](./author.md)                             | -                               |
| `role`                  | The role of the message in the conversation.                                  | [`MessageRole`](./message-role.md)                  | -                               |
| `content`               | The content of the message.                                                   | [`Block`](./block.md)*                              | -                               |
| `files`                 | The content of the message.                                                   | [`File`](./file.md)*                                | -                               |
| `isSelected`            | Whether this message is the selected message in the parent `ChatMessageGroup` | [`Boolean`](./boolean.md)                           | -                               |

# Related

The `ChatMessage` type is related to these types:

- Parents: [`Executable`](./executable.md)
- Children: none

# Bindings

The `ChatMessage` type is represented in:

- [JSON-LD](https://stencila.org/ChatMessage.jsonld)
- [JSON Schema](https://stencila.org/ChatMessage.schema.json)
- Python class [`ChatMessage`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ChatMessage`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/chat_message.rs)
- TypeScript class [`ChatMessage`](https://github.com/stencila/stencila/blob/main/ts/src/types/ChatMessage.ts)

***

This documentation was generated from [`ChatMessage.yaml`](https://github.com/stencila/stencila/blob/main/schema/ChatMessage.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
