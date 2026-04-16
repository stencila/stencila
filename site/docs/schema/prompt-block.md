---
title: Prompt Block
description: A preview of a rendered prompt at a location in a document.
---

This is an executable type used in Stencila Schema for previewing rendered prompts.

It exists to show how a prompt will resolve for a particular instruction type,
target node, or document position before or during an editing workflow. This
makes prompt selection and rendering inspectable within the document model
itself.

Key properties include `instructionType`, `nodeTypes`, `relativePosition`,
`query`, `target`, and rendered `content`.


This type is marked as unstable and is subject to change.

# Analogues

The following external types, elements, or nodes are similar to a `PromptBlock`:

- [rendered prompt preview](https://platform.openai.com/docs/guides/text-generation): Approximate analogue for inspecting the final prompt sent to a model, though Stencila models it as an executable document node tied to editing context.

# Properties

The `PromptBlock` type has these properties:

| Name                    | Description                                                      | Type                                                | Inherited from                  |
| ----------------------- | ---------------------------------------------------------------- | --------------------------------------------------- | ------------------------------- |
| `instructionType`       | The type of instruction the  being used for                      | [`InstructionType`](./instruction-type.md)          | -                               |
| `nodeTypes`             | The type of nodes the prompt is being used for                   | [`String`](./string.md)*                            | -                               |
| `relativePosition`      | The relative position of the node being edited, described etc.   | [`RelativePosition`](./relative-position.md)        | -                               |
| `query`                 | A user text query used to infer the `target` prompt              | [`String`](./string.md)                             | -                               |
| `target`                | An identifier for the prompt to be rendered                      | [`String`](./string.md)                             | -                               |
| `directory`             | The home directory of the prompt                                 | [`String`](./string.md)                             | -                               |
| `content`               | The executed content of the prompt                               | [`Block`](./block.md)*                              | -                               |
| `executionMode`         | Under which circumstances the node should be executed.           | [`ExecutionMode`](./execution-mode.md)              | [`Executable`](./executable.md) |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node. | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) |
| `compilationMessages`   | Messages generated while compiling the code.                     | [`CompilationMessage`](./compilation-message.md)*   | [`Executable`](./executable.md) |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.   | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) |
| `executionDependencies` | The upstream dependencies of this node.                          | [`ExecutionDependency`](./execution-dependency.md)* | [`Executable`](./executable.md) |
| `executionDependants`   | The downstream dependants of this node.                          | [`ExecutionDependant`](./execution-dependant.md)*   | [`Executable`](./executable.md) |
| `executionTags`         | Tags in the code which affect its execution.                     | [`ExecutionTag`](./execution-tag.md)*               | [`Executable`](./executable.md) |
| `executionCount`        | A count of the number of times that the node has been executed.  | [`Integer`](./integer.md)                           | [`Executable`](./executable.md) |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.   | [`ExecutionRequired`](./execution-required.md)      | [`Executable`](./executable.md) |
| `executionStatus`       | Status of the most recent, including any current, execution.     | [`ExecutionStatus`](./execution-status.md)          | [`Executable`](./executable.md) |
| `executionInstance`     | The id of the kernel instance that performed the last execution. | [`String`](./string.md)                             | [`Executable`](./executable.md) |
| `executionEnded`        | The timestamp when the last execution ended.                     | [`Timestamp`](./timestamp.md)                       | [`Executable`](./executable.md) |
| `executionDuration`     | Duration of the last execution.                                  | [`Duration`](./duration.md)                         | [`Executable`](./executable.md) |
| `executionMessages`     | Messages emitted while executing the node.                       | [`ExecutionMessage`](./execution-message.md)*       | [`Executable`](./executable.md) |
| `id`                    | The identifier for this item.                                    | [`String`](./string.md)                             | [`Entity`](./entity.md)         |

# Related

The `PromptBlock` type is related to these types:

- Parents: [`Executable`](./executable.md)
- Children: none

# Bindings

The `PromptBlock` type is represented in:

- [JSON-LD](https://stencila.org/PromptBlock.jsonld)
- [JSON Schema](https://stencila.org/PromptBlock.schema.json)
- Python class [`PromptBlock`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`PromptBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/prompt_block.rs)
- TypeScript class [`PromptBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/PromptBlock.ts)

***

This documentation was generated from [`PromptBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/PromptBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
