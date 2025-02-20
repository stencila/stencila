---
title: Executable
description: Abstract base type for executable nodes (e.g. `CodeChunk`, `CodeExpression`, `Call`).
config:
  publish:
    ghost:
      type: page
      slug: executable
      state: publish
      tags:
      - '#schema'
      - '#doc'
      - Flow
---

## Properties

The `Executable` type has these properties:

| Name                    | Description                                                      | Type                                                                                           | Inherited from                                                     | `JSON-LD @id`                        | Aliases                                                                                                                   |
| ----------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------- |
| `id`                    | The identifier for this item.                                    | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity) | [`schema:id`](https://schema.org/id) | -                                                                                                                         |
| `executionMode`         | Under which circumstances the node should be executed.           | [`ExecutionMode`](https://stencila.ghost.io/docs/reference/schema/execution-mode)              | -                                                                  | `stencila:executionMode`             | `execution-mode`, `execution_mode`                                                                                        |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node. | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)      | -                                                                  | `stencila:compilationDigest`         | `compilation-digest`, `compilation_digest`                                                                                |
| `compilationMessages`   | Messages generated while compiling the code.                     | [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message)*   | -                                                                  | `stencila:compilationMessages`       | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message`        |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.   | [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest)      | -                                                                  | `stencila:executionDigest`           | `execution-digest`, `execution_digest`                                                                                    |
| `executionDependencies` | The upstream dependencies of this node.                          | [`ExecutionDependency`](https://stencila.ghost.io/docs/reference/schema/execution-dependency)* | -                                                                  | `stencila:executionDependencies`     | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` |
| `executionDependants`   | The downstream dependants of this node.                          | [`ExecutionDependant`](https://stencila.ghost.io/docs/reference/schema/execution-dependant)*   | -                                                                  | `stencila:executionDependants`       | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        |
| `executionTags`         | Tags in the code which affect its execution.                     | [`ExecutionTag`](https://stencila.ghost.io/docs/reference/schema/execution-tag)*               | -                                                                  | `stencila:executionTags`             | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      |
| `executionCount`        | A count of the number of times that the node has been executed.  | [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)                           | -                                                                  | `stencila:executionCount`            | `execution-count`, `execution_count`                                                                                      |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.   | [`ExecutionRequired`](https://stencila.ghost.io/docs/reference/schema/execution-required)      | -                                                                  | `stencila:executionRequired`         | `execution-required`, `execution_required`                                                                                |
| `executionStatus`       | Status of the most recent, including any current, execution.     | [`ExecutionStatus`](https://stencila.ghost.io/docs/reference/schema/execution-status)          | -                                                                  | `stencila:executionStatus`           | `execution-status`, `execution_status`                                                                                    |
| `executionInstance`     | The id of the kernel instance that performed the last execution. | [`String`](https://stencila.ghost.io/docs/reference/schema/string)                             | -                                                                  | `stencila:executionInstance`         | `execution-instance`, `execution_instance`                                                                                |
| `executionEnded`        | The timestamp when the last execution ended.                     | [`Timestamp`](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | -                                                                  | `stencila:executionEnded`            | `execution-ended`, `execution_ended`                                                                                      |
| `executionDuration`     | Duration of the last execution.                                  | [`Duration`](https://stencila.ghost.io/docs/reference/schema/duration)                         | -                                                                  | `stencila:executionDuration`         | `execution-duration`, `execution_duration`                                                                                |
| `executionMessages`     | Messages emitted while executing the node.                       | [`ExecutionMessage`](https://stencila.ghost.io/docs/reference/schema/execution-message)*       | -                                                                  | `stencila:executionMessages`         | `execution-messages`, `execution_messages`, `executionMessage`, `execution-message`, `execution_message`                  |

## Related

The `Executable` type is related to these types:

- Parents: [`Entity`](https://stencila.ghost.io/docs/reference/schema/entity)
- Children: [`Article`](https://stencila.ghost.io/docs/reference/schema/article), [`Chat`](https://stencila.ghost.io/docs/reference/schema/chat), [`ChatMessage`](https://stencila.ghost.io/docs/reference/schema/chat-message), [`CodeExecutable`](https://stencila.ghost.io/docs/reference/schema/code-executable), [`Form`](https://stencila.ghost.io/docs/reference/schema/form), [`IfBlock`](https://stencila.ghost.io/docs/reference/schema/if-block), [`IncludeBlock`](https://stencila.ghost.io/docs/reference/schema/include-block), [`Instruction`](https://stencila.ghost.io/docs/reference/schema/instruction), [`Parameter`](https://stencila.ghost.io/docs/reference/schema/parameter), [`Prompt`](https://stencila.ghost.io/docs/reference/schema/prompt), [`PromptBlock`](https://stencila.ghost.io/docs/reference/schema/prompt-block)

## Bindings

The `Executable` type is represented in:

- [JSON-LD](https://stencila.org/Executable.jsonld)
- [JSON Schema](https://stencila.org/Executable.schema.json)
- Python class [`Executable`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/executable.py)
- Rust struct [`Executable`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/executable.rs)
- TypeScript class [`Executable`](https://github.com/stencila/stencila/blob/main/ts/src/types/Executable.ts)

## Source

This documentation was generated from [`Executable.yaml`](https://github.com/stencila/stencila/blob/main/schema/Executable.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
