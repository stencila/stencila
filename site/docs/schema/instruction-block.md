---
title: Instruction Block
description: An instruction to edit some block content.
---

# Properties

The `InstructionBlock` type has these properties:

| Name                    | Description                                                                | Type                                                | Inherited from                    |
| ----------------------- | -------------------------------------------------------------------------- | --------------------------------------------------- | --------------------------------- |
| `id`                    | The identifier for this item.                                              | [`String`](./string.md)                             | [`Entity`](./entity.md)           |
| `executionMode`         | Under which circumstances the node should be executed.                     | [`ExecutionMode`](./execution-mode.md)              | [`Executable`](./executable.md)   |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.           | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md)   |
| `compilationMessages`   | Messages generated while compiling the code.                               | [`CompilationMessage`](./compilation-message.md)*   | [`Executable`](./executable.md)   |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.             | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md)   |
| `executionDependencies` | The upstream dependencies of this node.                                    | [`ExecutionDependency`](./execution-dependency.md)* | [`Executable`](./executable.md)   |
| `executionDependants`   | The downstream dependants of this node.                                    | [`ExecutionDependant`](./execution-dependant.md)*   | [`Executable`](./executable.md)   |
| `executionTags`         | Tags in the code which affect its execution.                               | [`ExecutionTag`](./execution-tag.md)*               | [`Executable`](./executable.md)   |
| `executionCount`        | A count of the number of times that the node has been executed.            | [`Integer`](./integer.md)                           | [`Executable`](./executable.md)   |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.             | [`ExecutionRequired`](./execution-required.md)      | [`Executable`](./executable.md)   |
| `executionStatus`       | Status of the most recent, including any current, execution.               | [`ExecutionStatus`](./execution-status.md)          | [`Executable`](./executable.md)   |
| `executionInstance`     | The id of the kernel instance that performed the last execution.           | [`String`](./string.md)                             | [`Executable`](./executable.md)   |
| `executionEnded`        | The timestamp when the last execution ended.                               | [`Timestamp`](./timestamp.md)                       | [`Executable`](./executable.md)   |
| `executionDuration`     | Duration of the last execution.                                            | [`Duration`](./duration.md)                         | [`Executable`](./executable.md)   |
| `executionMessages`     | Messages emitted while executing the node.                                 | [`ExecutionMessage`](./execution-message.md)*       | [`Executable`](./executable.md)   |
| `instructionType`       | The type of instruction describing the operation to be performed.          | [`InstructionType`](./instruction-type.md)          | [`Instruction`](./instruction.md) |
| `prompt`                | The prompt selected, rendered and provided to the model                    | [`PromptBlock`](./prompt-block.md)                  | [`Instruction`](./instruction.md) |
| `message`               | The instruction message, possibly including images, audio, or other media. | [`InstructionMessage`](./instruction-message.md)    | [`Instruction`](./instruction.md) |
| `modelParameters`       | Model selection and inference parameters.                                  | [`ModelParameters`](./model-parameters.md)          | [`Instruction`](./instruction.md) |
| `activeSuggestion`      | The index of the suggestion that is currently active                       | [`UnsignedInteger`](./unsigned-integer.md)          | [`Instruction`](./instruction.md) |
| `content`               | The content to which the instruction applies.                              | [`Block`](./block.md)*                              | -                                 |
| `suggestions`           | Suggestions for the instruction                                            | [`SuggestionBlock`](./suggestion-block.md)*         | -                                 |

# Related

The `InstructionBlock` type is related to these types:

- Parents: [`Instruction`](./instruction.md)
- Children: none

# Bindings

The `InstructionBlock` type is represented in:

- [JSON-LD](https://stencila.org/InstructionBlock.jsonld)
- [JSON Schema](https://stencila.org/InstructionBlock.schema.json)
- Python class [`InstructionBlock`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`InstructionBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/instruction_block.rs)
- TypeScript class [`InstructionBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/InstructionBlock.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `InstructionBlock` type are generated using the following strategies.

::: table

| Property  | Complexity | Description                                               | Strategy                                  |
| --------- | ---------- | --------------------------------------------------------- | ----------------------------------------- |
| `content` | Min+       | No content                                                | `None`                                    |
|           | Low+       | Generate a single arbitrary, non-recursive, block node    | `option::of(vec_blocks_non_recursive(1))` |
|           | High+      | Generate up to two arbitrary, non-recursive, block nodes  | `option::of(vec_blocks_non_recursive(2))` |
|           | Max        | Generate up to four arbitrary, non-recursive, block nodes | `option::of(vec_blocks_non_recursive(4))` |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`InstructionBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/InstructionBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
