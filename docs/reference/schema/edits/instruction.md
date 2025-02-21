---
title: Instruction
description: Abstract base type for a document editing instruction.
config:
  publish:
    ghost:
      type: page
      slug: instruction
      state: publish
      tags:
      - '#schema'
      - '#doc'
      - Edits
---

## Properties

The `Instruction` type has these properties:

| Name                    | Description                                                                | Type                                                                                           | Inherited from                                                             | `JSON-LD @id`                        | Aliases                                                                                                                   |
| ----------------------- | -------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------- |
| `id`                    | The identifier for this item.                                              | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)         | [`schema:id`](https://schema.org/id) | -                                                                                                                         |
| `executionMode`         | Under which circumstances the node should be executed.                     | [`ExecutionMode`](https://stencila.ghost.io/docs/reference/schema/execution-mode)              | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionMode`             | `execution-mode`, `execution_mode`                                                                                        |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.           | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:compilationDigest`         | `compilation-digest`, `compilation_digest`                                                                                |
| `compilationMessages`   | Messages generated while compiling the code.                               | [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message)*   | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:compilationMessages`       | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message`        |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.             | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionDigest`           | `execution-digest`, `execution_digest`                                                                                    |
| `executionDependencies` | The upstream dependencies of this node.                                    | [`ExecutionDependency`](https://stencila.ghost.io/docs/reference/schema/execution-dependency)* | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionDependencies`     | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` |
| `executionDependants`   | The downstream dependants of this node.                                    | [`ExecutionDependant`](https://stencila.ghost.io/docs/reference/schema/execution-dependant)*   | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionDependants`       | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        |
| `executionTags`         | Tags in the code which affect its execution.                               | [`ExecutionTag`](https://stencila.ghost.io/docs/reference/schema/execution-tag)*               | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionTags`             | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      |
| `executionCount`        | A count of the number of times that the node has been executed.            | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)                           | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionCount`            | `execution-count`, `execution_count`                                                                                      |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.             | [`ExecutionRequired`](https://stencila.ghost.io/docs/reference/schema/execution-required)      | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionRequired`         | `execution-required`, `execution_required`                                                                                |
| `executionStatus`       | Status of the most recent, including any current, execution.               | [`ExecutionStatus`](https://stencila.ghost.io/docs/reference/schema/execution-status)          | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionStatus`           | `execution-status`, `execution_status`                                                                                    |
| `executionInstance`     | The id of the kernel instance that performed the last execution.           | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionInstance`         | `execution-instance`, `execution_instance`                                                                                |
| `executionEnded`        | The timestamp when the last execution ended.                               | [`Timestamp`](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionEnded`            | `execution-ended`, `execution_ended`                                                                                      |
| `executionDuration`     | Duration of the last execution.                                            | [`Duration`](https://stencila.ghost.io/docs/reference/schema/duration)                         | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionDuration`         | `execution-duration`, `execution_duration`                                                                                |
| `executionMessages`     | Messages emitted while executing the node.                                 | [`ExecutionMessage`](https://stencila.ghost.io/docs/reference/schema/execution-message)*       | [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable) | `stencila:executionMessages`         | `execution-messages`, `execution_messages`, `executionMessage`, `execution-message`, `execution_message`                  |
| `instructionType`       | The type of instruction describing the operation to be performed.          | [`InstructionType`](https://stencila.ghost.io/docs/reference/schema/instruction-type)          | -                                                                          | `stencila:instructionType`           | `instruction-type`, `instruction_type`                                                                                    |
| `prompt`                | The prompt selected, rendered and provided to the model                    | [`PromptBlock`](https://stencila.ghost.io/docs/reference/schema/prompt-block)                  | -                                                                          | `stencila:prompt`                    | -                                                                                                                         |
| `message`               | The instruction message, possibly including images, audio, or other media. | [`InstructionMessage`](https://stencila.ghost.io/docs/reference/schema/instruction-message)    | -                                                                          | `stencila:message`                   | -                                                                                                                         |
| `modelParameters`       | Model selection and inference parameters.                                  | [`ModelParameters`](https://stencila.ghost.io/docs/reference/schema/model-parameters)          | -                                                                          | `stencila:modelParameters`           | `model-parameters`, `model_parameters`, `model-params`, `model_params`, `model-pars`, `model_pars`, `model`               |
| `activeSuggestion`      | The index of the suggestion that is currently active                       | [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer)          | -                                                                          | `stencila:activeSuggestion`          | `active-suggestion`, `active_suggestion`                                                                                  |

## Related

The `Instruction` type is related to these types:

- Parents: [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable)
- Children: [`InstructionBlock`](https://stencila.ghost.io/docs/reference/schema/instruction-block), [`InstructionInline`](https://stencila.ghost.io/docs/reference/schema/instruction-inline)

## Bindings

The `Instruction` type is represented in:

- [JSON-LD](https://stencila.org/Instruction.jsonld)
- [JSON Schema](https://stencila.org/Instruction.schema.json)
- Python class [`Instruction`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/instruction.py)
- Rust struct [`Instruction`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/instruction.rs)
- TypeScript class [`Instruction`](https://github.com/stencila/stencila/blob/main/ts/src/types/Instruction.ts)

## Source

This documentation was generated from [`Instruction.yaml`](https://github.com/stencila/stencila/blob/main/schema/Instruction.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
