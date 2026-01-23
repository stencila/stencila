---
title: Form
description: A form to batch updates in document parameters.
---

This type is marked as experimental and is likely to change.

# Properties

The `Form` type has these properties:

| Name                    | Description                                                                               | Type                                                 | Inherited from                  |
| ----------------------- | ----------------------------------------------------------------------------------------- | ---------------------------------------------------- | ------------------------------- |
| `id`                    | The identifier for this item.                                                             | [`String`](./string.md)                              | [`Entity`](./entity.md)         |
| `executionMode`         | Under which circumstances the node should be executed.                                    | [`ExecutionMode`](./execution-mode.md)               | [`Executable`](./executable.md) |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.                          | [`CompilationDigest`](./compilation-digest.md)       | [`Executable`](./executable.md) |
| `compilationMessages`   | Messages generated while compiling the code.                                              | [`CompilationMessage`](./compilation-message.md)*    | [`Executable`](./executable.md) |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.                            | [`CompilationDigest`](./compilation-digest.md)       | [`Executable`](./executable.md) |
| `executionDependencies` | The upstream dependencies of this node.                                                   | [`ExecutionDependency`](./execution-dependency.md)*  | [`Executable`](./executable.md) |
| `executionDependants`   | The downstream dependants of this node.                                                   | [`ExecutionDependant`](./execution-dependant.md)*    | [`Executable`](./executable.md) |
| `executionTags`         | Tags in the code which affect its execution.                                              | [`ExecutionTag`](./execution-tag.md)*                | [`Executable`](./executable.md) |
| `executionCount`        | A count of the number of times that the node has been executed.                           | [`Integer`](./integer.md)                            | [`Executable`](./executable.md) |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.                            | [`ExecutionRequired`](./execution-required.md)       | [`Executable`](./executable.md) |
| `executionStatus`       | Status of the most recent, including any current, execution.                              | [`ExecutionStatus`](./execution-status.md)           | [`Executable`](./executable.md) |
| `executionInstance`     | The id of the kernel instance that performed the last execution.                          | [`String`](./string.md)                              | [`Executable`](./executable.md) |
| `executionEnded`        | The timestamp when the last execution ended.                                              | [`Timestamp`](./timestamp.md)                        | [`Executable`](./executable.md) |
| `executionDuration`     | Duration of the last execution.                                                           | [`Duration`](./duration.md)                          | [`Executable`](./executable.md) |
| `executionMessages`     | Messages emitted while executing the node.                                                | [`ExecutionMessage`](./execution-message.md)*        | [`Executable`](./executable.md) |
| `content`               | The content within the form, usually containing at least one `Parameter`.                 | [`Block`](./block.md)*                               | -                               |
| `deriveFrom`            | The dotted path to the object (e.g a database table) that the form should be derived from | [`String`](./string.md)                              | -                               |
| `deriveAction`          | The action (create, update or delete) to derive for the form                              | [`FormDeriveAction`](./form-derive-action.md)        | -                               |
| `deriveItem`            | An identifier for the item to be the target of Update or Delete actions                   | [`Integer`](./integer.md) \| [`String`](./string.md) | -                               |

# Related

The `Form` type is related to these types:

- Parents: [`Executable`](./executable.md)
- Children: none

# Bindings

The `Form` type is represented in:

- [JSON-LD](https://stencila.org/Form.jsonld)
- [JSON Schema](https://stencila.org/Form.schema.json)
- Python class [`Form`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Form`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/form.rs)
- TypeScript class [`Form`](https://github.com/stencila/stencila/blob/main/ts/src/types/Form.ts)

***

This documentation was generated from [`Form.yaml`](https://github.com/stencila/stencila/blob/main/schema/Form.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
