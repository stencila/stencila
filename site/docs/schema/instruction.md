---
title: Instruction
description: Abstract base type for a document editing instruction.
---

# Properties

The `Instruction` type has these properties:

| Name                    | Description                                                                | Type                                                | Inherited from                  | `JSON-LD @id`                        | Aliases                                                                                                                   |
| ----------------------- | -------------------------------------------------------------------------- | --------------------------------------------------- | ------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------- |
| `id`                    | The identifier for this item.                                              | [`String`](./string.md)                             | [`Entity`](./entity.md)         | [`schema:id`](https://schema.org/id) | -                                                                                                                         |
| `executionMode`         | Under which circumstances the node should be executed.                     | [`ExecutionMode`](./execution-mode.md)              | [`Executable`](./executable.md) | `stencila:executionMode`             | `execution-mode`, `execution_mode`                                                                                        |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.           | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) | `stencila:compilationDigest`         | `compilation-digest`, `compilation_digest`                                                                                |
| `compilationMessages`   | Messages generated while compiling the code.                               | [`CompilationMessage`](./compilation-message.md)*   | [`Executable`](./executable.md) | `stencila:compilationMessages`       | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message`        |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.             | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) | `stencila:executionDigest`           | `execution-digest`, `execution_digest`                                                                                    |
| `executionDependencies` | The upstream dependencies of this node.                                    | [`ExecutionDependency`](./execution-dependency.md)* | [`Executable`](./executable.md) | `stencila:executionDependencies`     | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` |
| `executionDependants`   | The downstream dependants of this node.                                    | [`ExecutionDependant`](./execution-dependant.md)*   | [`Executable`](./executable.md) | `stencila:executionDependants`       | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        |
| `executionTags`         | Tags in the code which affect its execution.                               | [`ExecutionTag`](./execution-tag.md)*               | [`Executable`](./executable.md) | `stencila:executionTags`             | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      |
| `executionCount`        | A count of the number of times that the node has been executed.            | [`Integer`](./integer.md)                           | [`Executable`](./executable.md) | `stencila:executionCount`            | `execution-count`, `execution_count`                                                                                      |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.             | [`ExecutionRequired`](./execution-required.md)      | [`Executable`](./executable.md) | `stencila:executionRequired`         | `execution-required`, `execution_required`                                                                                |
| `executionStatus`       | Status of the most recent, including any current, execution.               | [`ExecutionStatus`](./execution-status.md)          | [`Executable`](./executable.md) | `stencila:executionStatus`           | `execution-status`, `execution_status`                                                                                    |
| `executionInstance`     | The id of the kernel instance that performed the last execution.           | [`String`](./string.md)                             | [`Executable`](./executable.md) | `stencila:executionInstance`         | `execution-instance`, `execution_instance`                                                                                |
| `executionEnded`        | The timestamp when the last execution ended.                               | [`Timestamp`](./timestamp.md)                       | [`Executable`](./executable.md) | `stencila:executionEnded`            | `execution-ended`, `execution_ended`                                                                                      |
| `executionDuration`     | Duration of the last execution.                                            | [`Duration`](./duration.md)                         | [`Executable`](./executable.md) | `stencila:executionDuration`         | `execution-duration`, `execution_duration`                                                                                |
| `executionMessages`     | Messages emitted while executing the node.                                 | [`ExecutionMessage`](./execution-message.md)*       | [`Executable`](./executable.md) | `stencila:executionMessages`         | `execution-messages`, `execution_messages`, `executionMessage`, `execution-message`, `execution_message`                  |
| `instructionType`       | The type of instruction describing the operation to be performed.          | [`InstructionType`](./instruction-type.md)          | -                               | `stencila:instructionType`           | `instruction-type`, `instruction_type`                                                                                    |
| `prompt`                | The prompt selected, rendered and provided to the model                    | [`PromptBlock`](./prompt-block.md)                  | -                               | `stencila:prompt`                    | -                                                                                                                         |
| `message`               | The instruction message, possibly including images, audio, or other media. | [`InstructionMessage`](./instruction-message.md)    | -                               | `stencila:message`                   | -                                                                                                                         |
| `modelParameters`       | Model selection and inference parameters.                                  | [`ModelParameters`](./model-parameters.md)          | -                               | `stencila:modelParameters`           | `model-parameters`, `model_parameters`, `model-params`, `model_params`, `model-pars`, `model_pars`, `model`               |
| `activeSuggestion`      | The index of the suggestion that is currently active                       | [`UnsignedInteger`](./unsigned-integer.md)          | -                               | `stencila:activeSuggestion`          | `active-suggestion`, `active_suggestion`                                                                                  |

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
