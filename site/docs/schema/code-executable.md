---
title: Code Executable
description: An abstract base type for executable code nodes.
---

This is an abstract base type used in Stencila Schema for executable code nodes.

It extends [`Executable`](./executable.md) with the shared properties needed
for runnable code, such as source text, programming language, execution
bounds, and code-specific provenance. This provides a common foundation for
executable code blocks and expressions.

Key properties include `code`, `programmingLanguage`, `executionBounds`, and
inherited execution metadata from [`Executable`](./executable.md).


# Properties

The `CodeExecutable` type has these properties:

| Name                    | Description                                                      | Type                                                | Inherited from                  |
| ----------------------- | ---------------------------------------------------------------- | --------------------------------------------------- | ------------------------------- |
| `code`                  | The code.                                                        | [`Cord`](./cord.md)                                 | -                               |
| `programmingLanguage`   | The programming language of the code.                            | [`String`](./string.md)                             | -                               |
| `executionBounds`       | The environment in which code should be executed.                | [`ExecutionBounds`](./execution-bounds.md)          | -                               |
| `executionBounded`      | The execution bounds, if any, on the last execution.             | [`ExecutionBounds`](./execution-bounds.md)          | -                               |
| `authors`               | The authors of the executable code.                              | [`Author`](./author.md)*                            | -                               |
| `provenance`            | A summary of the provenance of the code.                         | [`ProvenanceCount`](./provenance-count.md)*         | -                               |
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

The `CodeExecutable` type is related to these types:

- Parents: [`Executable`](./executable.md)
- Children: [`Button`](./button.md), [`CodeChunk`](./code-chunk.md), [`CodeExpression`](./code-expression.md), [`ForBlock`](./for-block.md), [`IfBlockClause`](./if-block-clause.md)

# Bindings

The `CodeExecutable` type is represented in:

- [JSON-LD](https://stencila.org/CodeExecutable.jsonld)
- [JSON Schema](https://stencila.org/CodeExecutable.schema.json)
- Python class [`CodeExecutable`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`CodeExecutable`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_executable.rs)
- TypeScript class [`CodeExecutable`](https://github.com/stencila/stencila/blob/main/ts/src/types/CodeExecutable.ts)

***

This documentation was generated from [`CodeExecutable.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeExecutable.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
