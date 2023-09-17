---
title:
- type: Text
  value: CodeExecutable
---

# Code Executable

**Abstract base type for executable code nodes (e.g. `CodeChunk`).**

Adds properties to the base `Executable` node type that are necessary for executable code.
The added properties are the same as for static code nodes. Both `code` and `programmingLanguage` are required.


**`@id`**: `stencila:CodeExecutable`

## Properties

The `CodeExecutable` type has these properties:

| Name                  | `@id`                                                                  | Type                                                                                           | Description                                                                                       | Inherited from                                                                      |
| --------------------- | ---------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| id                    | [`schema:id`](https://schema.org/id)                                   | [`String`](https://stencila.dev/docs/reference/schema/data/string)                             | The identifier for this item                                                                      | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)                 |
| executionAuto         | `stencila:executionAuto`                                               | [`ExecutionAuto`](https://stencila.dev/docs/reference/schema/flow/execution-auto)              | Under which circumstances the code should be automatically executed.                              | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable)          |
| compilationDigest     | `stencila:compilationDigest`                                           | [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest)          | A digest of the content, semantics and dependencies of the node.                                  | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable)          |
| executionDigest       | `stencila:executionDigest`                                             | [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest)          | The `compileDigest` of the node when it was last executed.                                        | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable)          |
| executionDependencies | `stencila:executionDependencies`                                       | [`ExecutionDependency`](https://stencila.dev/docs/reference/schema/flow/execution-dependency)* | The upstream dependencies of this node.                                                           | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable)          |
| executionDependants   | `stencila:executionDependants`                                         | [`ExecutionDependant`](https://stencila.dev/docs/reference/schema/flow/execution-dependant)*   | The downstream dependants of this node.                                                           | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable)          |
| executionTags         | `stencila:executionTags`                                               | [`ExecutionTag`](https://stencila.dev/docs/reference/schema/flow/execution-tag)*               | Tags in the code which affect its execution                                                       | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable)          |
| executionCount        | `stencila:executionCount`                                              | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer)                           | A count of the number of times that the node has been executed.                                   | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable)          |
| executionRequired     | `stencila:executionRequired`                                           | [`ExecutionRequired`](https://stencila.dev/docs/reference/schema/flow/execution-required)      | Whether, and why, the code requires execution or re-execution.                                    | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable)          |
| executionKernel       | `stencila:executionKernel`                                             | [`String`](https://stencila.dev/docs/reference/schema/data/string)                             | The id of the kernel that the node was last executed in.                                          | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable)          |
| executionStatus       | `stencila:executionStatus`                                             | [`ExecutionStatus`](https://stencila.dev/docs/reference/schema/flow/execution-status)          | Status of the most recent, including any current, execution.                                      | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable)          |
| executionEnded        | `stencila:executionEnded`                                              | [`Timestamp`](https://stencila.dev/docs/reference/schema/data/timestamp)                       | The timestamp when the last execution ended.                                                      | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable)          |
| executionDuration     | `stencila:executionDuration`                                           | [`Duration`](https://stencila.dev/docs/reference/schema/data/duration)                         | Duration of the last execution.                                                                   | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable)          |
| errors                | `stencila:errors`                                                      | [`CodeError`](https://stencila.dev/docs/reference/schema/code/code-error)*                     | Errors when compiling (e.g. syntax errors) or executing the node.                                 | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable)          |
| code                  | `stencila:code`                                                        | [`Cord`](https://stencila.dev/docs/reference/schema/data/cord)                                 | The code.                                                                                         | [`CodeExecutable`](https://stencila.dev/docs/reference/schema/code/code-executable) |
| programmingLanguage   | [`schema:programmingLanguage`](https://schema.org/programmingLanguage) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                             | The programming language of the code.                                                             | [`CodeExecutable`](https://stencila.dev/docs/reference/schema/code/code-executable) |
| guessLanguage         | `stencila:guessLanguage`                                               | [`Boolean`](https://stencila.dev/docs/reference/schema/data/boolean)                           | Whether the programming language of the code should be guessed based on syntax and variables used | [`CodeExecutable`](https://stencila.dev/docs/reference/schema/code/code-executable) |

## Related

The `CodeExecutable` type is related to these types:

- Parents: [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable)
- Children: [`Button`](https://stencila.dev/docs/reference/schema/flow/button), [`CodeChunk`](https://stencila.dev/docs/reference/schema/code/code-chunk), [`CodeExpression`](https://stencila.dev/docs/reference/schema/code/code-expression), [`For`](https://stencila.dev/docs/reference/schema/flow/for), [`IfClause`](https://stencila.dev/docs/reference/schema/flow/if-clause)

## Bindings

The `CodeExecutable` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/CodeExecutable.jsonld)
- [JSON Schema](https://stencila.dev/CodeExecutable.schema.json)
- Python class [`CodeExecutable`](https://github.com/stencila/stencila/blob/main/python/stencila/types/code_executable.py)
- Rust struct [`CodeExecutable`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/code_executable.rs)
- TypeScript class [`CodeExecutable`](https://github.com/stencila/stencila/blob/main/typescript/src/types/CodeExecutable.ts)

## Source

This documentation was generated from [`CodeExecutable.yaml`](https://github.com/stencila/stencila/blob/main/schema/CodeExecutable.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).