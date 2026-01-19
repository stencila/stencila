---
title: Executable
description: Abstract base type for executable nodes (e.g. `CodeChunk`, `CodeExpression`, `Call`).
---

# Properties

The `Executable` type has these properties:

| Name                    | Description                                                      | Type                                                | Inherited from          | `JSON-LD @id`                        | Aliases                                                                                                                   |
| ----------------------- | ---------------------------------------------------------------- | --------------------------------------------------- | ----------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------- |
| `id`                    | The identifier for this item.                                    | [`String`](./string.md)                             | [`Entity`](./entity.md) | [`schema:id`](https://schema.org/id) | -                                                                                                                         |
| `executionMode`         | Under which circumstances the node should be executed.           | [`ExecutionMode`](./execution-mode.md)              | -                       | `stencila:executionMode`             | `execution-mode`, `execution_mode`                                                                                        |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node. | [`CompilationDigest`](./compilation-digest.md)      | -                       | `stencila:compilationDigest`         | `compilation-digest`, `compilation_digest`                                                                                |
| `compilationMessages`   | Messages generated while compiling the code.                     | [`CompilationMessage`](./compilation-message.md)*   | -                       | `stencila:compilationMessages`       | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message`        |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.   | [`CompilationDigest`](./compilation-digest.md)      | -                       | `stencila:executionDigest`           | `execution-digest`, `execution_digest`                                                                                    |
| `executionDependencies` | The upstream dependencies of this node.                          | [`ExecutionDependency`](./execution-dependency.md)* | -                       | `stencila:executionDependencies`     | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` |
| `executionDependants`   | The downstream dependants of this node.                          | [`ExecutionDependant`](./execution-dependant.md)*   | -                       | `stencila:executionDependants`       | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        |
| `executionTags`         | Tags in the code which affect its execution.                     | [`ExecutionTag`](./execution-tag.md)*               | -                       | `stencila:executionTags`             | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      |
| `executionCount`        | A count of the number of times that the node has been executed.  | [`Integer`](./integer.md)                           | -                       | `stencila:executionCount`            | `execution-count`, `execution_count`                                                                                      |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.   | [`ExecutionRequired`](./execution-required.md)      | -                       | `stencila:executionRequired`         | `execution-required`, `execution_required`                                                                                |
| `executionStatus`       | Status of the most recent, including any current, execution.     | [`ExecutionStatus`](./execution-status.md)          | -                       | `stencila:executionStatus`           | `execution-status`, `execution_status`                                                                                    |
| `executionInstance`     | The id of the kernel instance that performed the last execution. | [`String`](./string.md)                             | -                       | `stencila:executionInstance`         | `execution-instance`, `execution_instance`                                                                                |
| `executionEnded`        | The timestamp when the last execution ended.                     | [`Timestamp`](./timestamp.md)                       | -                       | `stencila:executionEnded`            | `execution-ended`, `execution_ended`                                                                                      |
| `executionDuration`     | Duration of the last execution.                                  | [`Duration`](./duration.md)                         | -                       | `stencila:executionDuration`         | `execution-duration`, `execution_duration`                                                                                |
| `executionMessages`     | Messages emitted while executing the node.                       | [`ExecutionMessage`](./execution-message.md)*       | -                       | `stencila:executionMessages`         | `execution-messages`, `execution_messages`, `executionMessage`, `execution-message`, `execution_message`                  |

# Related

The `Executable` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: [`Article`](./article.md), [`Bibliography`](./bibliography.md), [`Chat`](./chat.md), [`ChatMessage`](./chat-message.md), [`CodeExecutable`](./code-executable.md), [`Form`](./form.md), [`IfBlock`](./if-block.md), [`IncludeBlock`](./include-block.md), [`Instruction`](./instruction.md), [`Parameter`](./parameter.md), [`Prompt`](./prompt.md), [`PromptBlock`](./prompt-block.md)

# Bindings

The `Executable` type is represented in:

- [JSON-LD](https://stencila.org/Executable.jsonld)
- [JSON Schema](https://stencila.org/Executable.schema.json)
- Python class [`Executable`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/executable.py)
- Rust struct [`Executable`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/executable.rs)
- TypeScript class [`Executable`](https://github.com/stencila/stencila/blob/main/ts/src/types/Executable.ts)

# Source

This documentation was generated from [`Executable.yaml`](https://github.com/stencila/stencila/blob/main/schema/Executable.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
