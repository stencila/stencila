---
title: Executable
description: An abstract base type for executable nodes.
---

This is an abstract base type used in Stencila Schema for nodes that can be compiled
or executed.

It adds execution state, dependency tracking, digests, messages, and timing
information to the core document node model. This makes executable behavior a
first-class capability that can be shared by code, prompts, chats,
instructions, forms, and other interactive document nodes.

Key properties include `executionMode`, `compilationDigest`,
`executionDependencies`, `executionRequired`, `executionStatus`,
`executionMessages`, and related execution timestamps and durations.


# Analogues

The following external types, elements, or nodes are similar to a `Executable`:

- [notebook executable cell model](https://nbformat.readthedocs.io/): Approximate analogue for document nodes that carry execution state, though Stencila generalizes execution semantics beyond code cells to many node kinds.

# Properties

The `Executable` type has these properties:

| Name                    | Description                                                      | Type                                                | Inherited from          |
| ----------------------- | ---------------------------------------------------------------- | --------------------------------------------------- | ----------------------- |
| `executionMode`         | Under which circumstances the node should be executed.           | [`ExecutionMode`](./execution-mode.md)              | -                       |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node. | [`CompilationDigest`](./compilation-digest.md)      | -                       |
| `compilationMessages`   | Messages generated while compiling the code.                     | [`CompilationMessage`](./compilation-message.md)*   | -                       |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.   | [`CompilationDigest`](./compilation-digest.md)      | -                       |
| `executionDependencies` | The upstream dependencies of this node.                          | [`ExecutionDependency`](./execution-dependency.md)* | -                       |
| `executionDependants`   | The downstream dependants of this node.                          | [`ExecutionDependant`](./execution-dependant.md)*   | -                       |
| `executionTags`         | Tags in the code which affect its execution.                     | [`ExecutionTag`](./execution-tag.md)*               | -                       |
| `executionCount`        | A count of the number of times that the node has been executed.  | [`Integer`](./integer.md)                           | -                       |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.   | [`ExecutionRequired`](./execution-required.md)      | -                       |
| `executionStatus`       | Status of the most recent, including any current, execution.     | [`ExecutionStatus`](./execution-status.md)          | -                       |
| `executionInstance`     | The id of the kernel instance that performed the last execution. | [`String`](./string.md)                             | -                       |
| `executionEnded`        | The timestamp when the last execution ended.                     | [`Timestamp`](./timestamp.md)                       | -                       |
| `executionDuration`     | Duration of the last execution.                                  | [`Duration`](./duration.md)                         | -                       |
| `executionMessages`     | Messages emitted while executing the node.                       | [`ExecutionMessage`](./execution-message.md)*       | -                       |
| `id`                    | The identifier for this item.                                    | [`String`](./string.md)                             | [`Entity`](./entity.md) |

# Related

The `Executable` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: [`Article`](./article.md), [`Bibliography`](./bibliography.md), [`Chat`](./chat.md), [`ChatMessage`](./chat-message.md), [`CodeExecutable`](./code-executable.md), [`Form`](./form.md), [`IfBlock`](./if-block.md), [`IncludeBlock`](./include-block.md), [`Instruction`](./instruction.md), [`Parameter`](./parameter.md), [`Prompt`](./prompt.md), [`PromptBlock`](./prompt-block.md)

# Bindings

The `Executable` type is represented in:

- [JSON-LD](https://stencila.org/Executable.jsonld)
- [JSON Schema](https://stencila.org/Executable.schema.json)
- Python class [`Executable`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Executable`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/executable.rs)
- TypeScript class [`Executable`](https://github.com/stencila/stencila/blob/main/ts/src/types/Executable.ts)

***

This documentation was generated from [`Executable.yaml`](https://github.com/stencila/stencila/blob/main/schema/Executable.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
