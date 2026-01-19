---
title: Code Executable
description: Abstract base type for executable code nodes (e.g. `CodeChunk`).
---

Adds properties to the base `Executable` node type that are necessary for executable code.
The added properties are the same as for static code nodes. Both `code` and `programmingLanguage` are required.


# Properties

The `CodeExecutable` type has these properties:

| Name                    | Description                                                      | Type                                                | Inherited from                  | `JSON-LD @id`                                                          | Aliases                                                                                                                   |
| ----------------------- | ---------------------------------------------------------------- | --------------------------------------------------- | ------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `id`                    | The identifier for this item.                                    | [`String`](./string.md)                             | [`Entity`](./entity.md)         | [`schema:id`](https://schema.org/id)                                   | -                                                                                                                         |
| `executionMode`         | Under which circumstances the node should be executed.           | [`ExecutionMode`](./execution-mode.md)              | [`Executable`](./executable.md) | `stencila:executionMode`                                               | `execution-mode`, `execution_mode`                                                                                        |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node. | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) | `stencila:compilationDigest`                                           | `compilation-digest`, `compilation_digest`                                                                                |
| `compilationMessages`   | Messages generated while compiling the code.                     | [`CompilationMessage`](./compilation-message.md)*   | [`Executable`](./executable.md) | `stencila:compilationMessages`                                         | `compilation-messages`, `compilation_messages`, `compilationMessage`, `compilation-message`, `compilation_message`        |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.   | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) | `stencila:executionDigest`                                             | `execution-digest`, `execution_digest`                                                                                    |
| `executionDependencies` | The upstream dependencies of this node.                          | [`ExecutionDependency`](./execution-dependency.md)* | [`Executable`](./executable.md) | `stencila:executionDependencies`                                       | `execution-dependencies`, `execution_dependencies`, `executionDependency`, `execution-dependency`, `execution_dependency` |
| `executionDependants`   | The downstream dependants of this node.                          | [`ExecutionDependant`](./execution-dependant.md)*   | [`Executable`](./executable.md) | `stencila:executionDependants`                                         | `execution-dependants`, `execution_dependants`, `executionDependant`, `execution-dependant`, `execution_dependant`        |
| `executionTags`         | Tags in the code which affect its execution.                     | [`ExecutionTag`](./execution-tag.md)*               | [`Executable`](./executable.md) | `stencila:executionTags`                                               | `execution-tags`, `execution_tags`, `executionTag`, `execution-tag`, `execution_tag`                                      |
| `executionCount`        | A count of the number of times that the node has been executed.  | [`Integer`](./integer.md)                           | [`Executable`](./executable.md) | `stencila:executionCount`                                              | `execution-count`, `execution_count`                                                                                      |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.   | [`ExecutionRequired`](./execution-required.md)      | [`Executable`](./executable.md) | `stencila:executionRequired`                                           | `execution-required`, `execution_required`                                                                                |
| `executionStatus`       | Status of the most recent, including any current, execution.     | [`ExecutionStatus`](./execution-status.md)          | [`Executable`](./executable.md) | `stencila:executionStatus`                                             | `execution-status`, `execution_status`                                                                                    |
| `executionInstance`     | The id of the kernel instance that performed the last execution. | [`String`](./string.md)                             | [`Executable`](./executable.md) | `stencila:executionInstance`                                           | `execution-instance`, `execution_instance`                                                                                |
| `executionEnded`        | The timestamp when the last execution ended.                     | [`Timestamp`](./timestamp.md)                       | [`Executable`](./executable.md) | `stencila:executionEnded`                                              | `execution-ended`, `execution_ended`                                                                                      |
| `executionDuration`     | Duration of the last execution.                                  | [`Duration`](./duration.md)                         | [`Executable`](./executable.md) | `stencila:executionDuration`                                           | `execution-duration`, `execution_duration`                                                                                |
| `executionMessages`     | Messages emitted while executing the node.                       | [`ExecutionMessage`](./execution-message.md)*       | [`Executable`](./executable.md) | `stencila:executionMessages`                                           | `execution-messages`, `execution_messages`, `executionMessage`, `execution-message`, `execution_message`                  |
| `code`                  | The code.                                                        | [`Cord`](./cord.md)                                 | -                               | `stencila:code`                                                        | -                                                                                                                         |
| `programmingLanguage`   | The programming language of the code.                            | [`String`](./string.md)                             | -                               | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | `programming-language`, `programming_language`                                                                            |
| `executionBounds`       | The environment in which code should be executed.                | [`ExecutionBounds`](./execution-bounds.md)          | -                               | `stencila:executionBounds`                                             | `execution-bounds`, `execution_bounds`                                                                                    |
| `executionBounded`      | The execution bounds, if any, on the last execution.             | [`ExecutionBounds`](./execution-bounds.md)          | -                               | `stencila:executionBounded`                                            | `execution-bounded`, `execution_bounded`                                                                                  |
| `authors`               | The authors of the executable code.                              | [`Author`](./author.md)*                            | -                               | [`schema:author`](https://schema.org/author)                           | `author`                                                                                                                  |
| `provenance`            | A summary of the provenance of the code.                         | [`ProvenanceCount`](./provenance-count.md)*         | -                               | `stencila:provenance`                                                  | -                                                                                                                         |

# Related

The `CodeExecutable` type is related to these types:

- Parents: [`Executable`](./executable.md)
- Children: [`Button`](./button.md), [`CodeChunk`](./code-chunk.md), [`CodeExpression`](./code-expression.md), [`ForBlock`](./for-block.md), [`IfBlockClause`](./if-block-clause.md)

# Bindings

The `CodeExecutable` type is represented in:

- [JSON-LD](https://stencila.org/CodeExecutable.jsonld)
- [JSON Schema](https://stencila.org/CodeExecutable.schema.json)
- Python class [`CodeExecutable`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/code_executable.py)
- Rust struct [`CodeExecutable`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_executable.rs)
- TypeScript class [`CodeExecutable`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeExecutable.ts)

# Source

This documentation was generated from [`CodeExecutable.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeExecutable.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
