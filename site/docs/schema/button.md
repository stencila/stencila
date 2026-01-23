---
title: Button
description: A button.
---

This type is marked as experimental and is likely to change.

# Properties

The `Button` type has these properties:

| Name                    | Description                                                      | Type                                                | Inherited from                           |
| ----------------------- | ---------------------------------------------------------------- | --------------------------------------------------- | ---------------------------------------- |
| `id`                    | The identifier for this item.                                    | [`String`](./string.md)                             | [`Entity`](./entity.md)                  |
| `executionMode`         | Under which circumstances the node should be executed.           | [`ExecutionMode`](./execution-mode.md)              | [`Executable`](./executable.md)          |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node. | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md)          |
| `compilationMessages`   | Messages generated while compiling the code.                     | [`CompilationMessage`](./compilation-message.md)*   | [`Executable`](./executable.md)          |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.   | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md)          |
| `executionDependencies` | The upstream dependencies of this node.                          | [`ExecutionDependency`](./execution-dependency.md)* | [`Executable`](./executable.md)          |
| `executionDependants`   | The downstream dependants of this node.                          | [`ExecutionDependant`](./execution-dependant.md)*   | [`Executable`](./executable.md)          |
| `executionTags`         | Tags in the code which affect its execution.                     | [`ExecutionTag`](./execution-tag.md)*               | [`Executable`](./executable.md)          |
| `executionCount`        | A count of the number of times that the node has been executed.  | [`Integer`](./integer.md)                           | [`Executable`](./executable.md)          |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.   | [`ExecutionRequired`](./execution-required.md)      | [`Executable`](./executable.md)          |
| `executionStatus`       | Status of the most recent, including any current, execution.     | [`ExecutionStatus`](./execution-status.md)          | [`Executable`](./executable.md)          |
| `executionInstance`     | The id of the kernel instance that performed the last execution. | [`String`](./string.md)                             | [`Executable`](./executable.md)          |
| `executionEnded`        | The timestamp when the last execution ended.                     | [`Timestamp`](./timestamp.md)                       | [`Executable`](./executable.md)          |
| `executionDuration`     | Duration of the last execution.                                  | [`Duration`](./duration.md)                         | [`Executable`](./executable.md)          |
| `executionMessages`     | Messages emitted while executing the node.                       | [`ExecutionMessage`](./execution-message.md)*       | [`Executable`](./executable.md)          |
| `code`                  | The code.                                                        | [`Cord`](./cord.md)                                 | [`CodeExecutable`](./code-executable.md) |
| `programmingLanguage`   | The programming language of the code.                            | [`String`](./string.md)                             | [`CodeExecutable`](./code-executable.md) |
| `executionBounds`       | The environment in which code should be executed.                | [`ExecutionBounds`](./execution-bounds.md)          | [`CodeExecutable`](./code-executable.md) |
| `executionBounded`      | The execution bounds, if any, on the last execution.             | [`ExecutionBounds`](./execution-bounds.md)          | [`CodeExecutable`](./code-executable.md) |
| `authors`               | The authors of the executable code.                              | [`Author`](./author.md)*                            | [`CodeExecutable`](./code-executable.md) |
| `provenance`            | A summary of the provenance of the code.                         | [`ProvenanceCount`](./provenance-count.md)*         | [`CodeExecutable`](./code-executable.md) |
| `name`                  | The name of the variable associated with the button.             | [`String`](./string.md)                             | -                                        |
| `label`                 | A label for the button                                           | [`String`](./string.md)                             | -                                        |
| `isDisabled`            | Whether the button is currently disabled                         | [`Boolean`](./boolean.md)                           | -                                        |

# Related

The `Button` type is related to these types:

- Parents: [`CodeExecutable`](./code-executable.md)
- Children: none

# Bindings

The `Button` type is represented in:

- [JSON-LD](https://stencila.org/Button.jsonld)
- [JSON Schema](https://stencila.org/Button.schema.json)
- Python class [`Button`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Button`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/button.rs)
- TypeScript class [`Button`](https://github.com/stencila/stencila/blob/main/ts/src/types/Button.ts)

***

This documentation was generated from [`Button.yaml`](https://github.com/stencila/stencila/blob/main/schema/Button.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
