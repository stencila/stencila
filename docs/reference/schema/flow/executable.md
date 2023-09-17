---
title:
- type: Text
  value: Executable
---

# Executable

**Abstract base type for executable nodes (e.g. `CodeChunk`, `CodeExpression`, `Call`).**

**`@id`**: `stencila:Executable`

## Properties

The `Executable` type has these properties:

| Name                  | `@id`                                | Type                                                                                           | Description                                                          | Inherited from                                                             |
| --------------------- | ------------------------------------ | ---------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| id                    | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string)                             | The identifier for this item                                         | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)        |
| executionAuto         | `stencila:executionAuto`             | [`ExecutionAuto`](https://stencila.dev/docs/reference/schema/flow/execution-auto)              | Under which circumstances the code should be automatically executed. | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable) |
| compilationDigest     | `stencila:compilationDigest`         | [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest)          | A digest of the content, semantics and dependencies of the node.     | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable) |
| executionDigest       | `stencila:executionDigest`           | [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest)          | The `compileDigest` of the node when it was last executed.           | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable) |
| executionDependencies | `stencila:executionDependencies`     | [`ExecutionDependency`](https://stencila.dev/docs/reference/schema/flow/execution-dependency)* | The upstream dependencies of this node.                              | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable) |
| executionDependants   | `stencila:executionDependants`       | [`ExecutionDependant`](https://stencila.dev/docs/reference/schema/flow/execution-dependant)*   | The downstream dependants of this node.                              | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable) |
| executionTags         | `stencila:executionTags`             | [`ExecutionTag`](https://stencila.dev/docs/reference/schema/flow/execution-tag)*               | Tags in the code which affect its execution                          | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable) |
| executionCount        | `stencila:executionCount`            | [`Integer`](https://stencila.dev/docs/reference/schema/data/integer)                           | A count of the number of times that the node has been executed.      | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable) |
| executionRequired     | `stencila:executionRequired`         | [`ExecutionRequired`](https://stencila.dev/docs/reference/schema/flow/execution-required)      | Whether, and why, the code requires execution or re-execution.       | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable) |
| executionKernel       | `stencila:executionKernel`           | [`String`](https://stencila.dev/docs/reference/schema/data/string)                             | The id of the kernel that the node was last executed in.             | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable) |
| executionStatus       | `stencila:executionStatus`           | [`ExecutionStatus`](https://stencila.dev/docs/reference/schema/flow/execution-status)          | Status of the most recent, including any current, execution.         | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable) |
| executionEnded        | `stencila:executionEnded`            | [`Timestamp`](https://stencila.dev/docs/reference/schema/data/timestamp)                       | The timestamp when the last execution ended.                         | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable) |
| executionDuration     | `stencila:executionDuration`         | [`Duration`](https://stencila.dev/docs/reference/schema/data/duration)                         | Duration of the last execution.                                      | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable) |
| errors                | `stencila:errors`                    | [`CodeError`](https://stencila.dev/docs/reference/schema/code/code-error)*                     | Errors when compiling (e.g. syntax errors) or executing the node.    | [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable) |

## Related

The `Executable` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: [`CodeExecutable`](https://stencila.dev/docs/reference/schema/code/code-executable), [`Form`](https://stencila.dev/docs/reference/schema/flow/form), [`If`](https://stencila.dev/docs/reference/schema/flow/if), [`Include`](https://stencila.dev/docs/reference/schema/flow/include), [`Parameter`](https://stencila.dev/docs/reference/schema/flow/parameter)

## Bindings

The `Executable` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Executable.jsonld)
- [JSON Schema](https://stencila.dev/Executable.schema.json)
- Python class [`Executable`](https://github.com/stencila/stencila/blob/main/python/stencila/types/executable.py)
- Rust struct [`Executable`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/executable.rs)
- TypeScript class [`Executable`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Executable.ts)

## Source

This documentation was generated from [`Executable.yaml`](https://github.com/stencila/stencila/blob/main/schema/Executable.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).