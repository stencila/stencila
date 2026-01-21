---
title: Instruction
description: Abstract base type for a document editing instruction.
---

# Properties

The `Instruction` type has these properties:

| Name                    | Description                                                                | Type                                                | Inherited from                  |
| ----------------------- | -------------------------------------------------------------------------- | --------------------------------------------------- | ------------------------------- |
| `id`                    | The identifier for this item.                                              | [`String`](./string.md)                             | [`Entity`](./entity.md)         |
| `executionMode`         | Under which circumstances the node should be executed.                     | [`ExecutionMode`](./execution-mode.md)              | [`Executable`](./executable.md) |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.           | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) |
| `compilationMessages`   | Messages generated while compiling the code.                               | [`CompilationMessage`](./compilation-message.md)*   | [`Executable`](./executable.md) |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.             | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) |
| `executionDependencies` | The upstream dependencies of this node.                                    | [`ExecutionDependency`](./execution-dependency.md)* | [`Executable`](./executable.md) |
| `executionDependants`   | The downstream dependants of this node.                                    | [`ExecutionDependant`](./execution-dependant.md)*   | [`Executable`](./executable.md) |
| `executionTags`         | Tags in the code which affect its execution.                               | [`ExecutionTag`](./execution-tag.md)*               | [`Executable`](./executable.md) |
| `executionCount`        | A count of the number of times that the node has been executed.            | [`Integer`](./integer.md)                           | [`Executable`](./executable.md) |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.             | [`ExecutionRequired`](./execution-required.md)      | [`Executable`](./executable.md) |
| `executionStatus`       | Status of the most recent, including any current, execution.               | [`ExecutionStatus`](./execution-status.md)          | [`Executable`](./executable.md) |
| `executionInstance`     | The id of the kernel instance that performed the last execution.           | [`String`](./string.md)                             | [`Executable`](./executable.md) |
| `executionEnded`        | The timestamp when the last execution ended.                               | [`Timestamp`](./timestamp.md)                       | [`Executable`](./executable.md) |
| `executionDuration`     | Duration of the last execution.                                            | [`Duration`](./duration.md)                         | [`Executable`](./executable.md) |
| `executionMessages`     | Messages emitted while executing the node.                                 | [`ExecutionMessage`](./execution-message.md)*       | [`Executable`](./executable.md) |
| `instructionType`       | The type of instruction describing the operation to be performed.          | [`InstructionType`](./instruction-type.md)          | -                               |
| `prompt`                | The prompt selected, rendered and provided to the model                    | [`PromptBlock`](./prompt-block.md)                  | -                               |
| `message`               | The instruction message, possibly including images, audio, or other media. | [`InstructionMessage`](./instruction-message.md)    | -                               |
| `modelParameters`       | Model selection and inference parameters.                                  | [`ModelParameters`](./model-parameters.md)          | -                               |
| `activeSuggestion`      | The index of the suggestion that is currently active                       | [`UnsignedInteger`](./unsigned-integer.md)          | -                               |

# Related

The `Instruction` type is related to these types:

- Parents: [`Executable`](./executable.md)
- Children: [`InstructionBlock`](./instruction-block.md), [`InstructionInline`](./instruction-inline.md)

# Bindings

The `Instruction` type is represented in:

- [JSON-LD](https://stencila.org/Instruction.jsonld)
- [JSON Schema](https://stencila.org/Instruction.schema.json)
- Python class [`Instruction`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/instruction.py)
- Rust struct [`Instruction`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/instruction.rs)
- TypeScript class [`Instruction`](https://github.com/stencila/stencila/blob/main/ts/src/types/Instruction.ts)

# Source

This documentation was generated from [`Instruction.yaml`](https://github.com/stencila/stencila/blob/main/schema/Instruction.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
