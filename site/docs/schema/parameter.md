---
title: Parameter
description: A parameter of a document.
---

This type is marked as unstable and is subject to change.

# Properties

The `Parameter` type has these properties:

| Name                    | Description                                                                                            | Type                                                | Inherited from                  |
| ----------------------- | ------------------------------------------------------------------------------------------------------ | --------------------------------------------------- | ------------------------------- |
| `id`                    | The identifier for this item.                                                                          | [`String`](./string.md)                             | [`Entity`](./entity.md)         |
| `executionMode`         | Under which circumstances the node should be executed.                                                 | [`ExecutionMode`](./execution-mode.md)              | [`Executable`](./executable.md) |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.                                       | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) |
| `compilationMessages`   | Messages generated while compiling the code.                                                           | [`CompilationMessage`](./compilation-message.md)*   | [`Executable`](./executable.md) |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.                                         | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md) |
| `executionDependencies` | The upstream dependencies of this node.                                                                | [`ExecutionDependency`](./execution-dependency.md)* | [`Executable`](./executable.md) |
| `executionDependants`   | The downstream dependants of this node.                                                                | [`ExecutionDependant`](./execution-dependant.md)*   | [`Executable`](./executable.md) |
| `executionTags`         | Tags in the code which affect its execution.                                                           | [`ExecutionTag`](./execution-tag.md)*               | [`Executable`](./executable.md) |
| `executionCount`        | A count of the number of times that the node has been executed.                                        | [`Integer`](./integer.md)                           | [`Executable`](./executable.md) |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.                                         | [`ExecutionRequired`](./execution-required.md)      | [`Executable`](./executable.md) |
| `executionStatus`       | Status of the most recent, including any current, execution.                                           | [`ExecutionStatus`](./execution-status.md)          | [`Executable`](./executable.md) |
| `executionInstance`     | The id of the kernel instance that performed the last execution.                                       | [`String`](./string.md)                             | [`Executable`](./executable.md) |
| `executionEnded`        | The timestamp when the last execution ended.                                                           | [`Timestamp`](./timestamp.md)                       | [`Executable`](./executable.md) |
| `executionDuration`     | Duration of the last execution.                                                                        | [`Duration`](./duration.md)                         | [`Executable`](./executable.md) |
| `executionMessages`     | Messages emitted while executing the node.                                                             | [`ExecutionMessage`](./execution-message.md)*       | [`Executable`](./executable.md) |
| `name`                  | The name of the parameter.                                                                             | [`String`](./string.md)                             | -                               |
| `label`                 | A short label for the parameter.                                                                       | [`String`](./string.md)                             | -                               |
| `value`                 | The current value of the parameter.                                                                    | [`Node`](./node.md)                                 | -                               |
| `default`               | The default value of the parameter.                                                                    | [`Node`](./node.md)                                 | -                               |
| `validator`             | The validator that the value is validated against.                                                     | [`Validator`](./validator.md)                       | -                               |
| `derivedFrom`           | The dotted path to the object (e.g. a database table column) that the parameter should be derived from | [`String`](./string.md)                             | -                               |

# Related

The `Parameter` type is related to these types:

- Parents: [`Executable`](./executable.md)
- Children: [`CallArgument`](./call-argument.md)

# Bindings

The `Parameter` type is represented in:

- [JSON-LD](https://stencila.org/Parameter.jsonld)
- [JSON Schema](https://stencila.org/Parameter.schema.json)
- Python class [`Parameter`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/parameter.py)
- Rust struct [`Parameter`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/parameter.rs)
- TypeScript class [`Parameter`](https://github.com/stencila/stencila/blob/main/ts/src/types/Parameter.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `Parameter` type are generated using the following strategies.

::: table

| Property | Complexity | Description                                                                                                                               | Strategy                          |
| -------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------- |
| `name`   | Min+       | Generate a fixed name.                                                                                                                    | `String::from("name")`            |
|          | Low+       | Generate a random string of up to 10 alphanumeric characters (and at most one underscore to avoid<br><br>a clash with Markdown emphasis). | Regex `[a-zA-Z_][a-zA-Z0-9]{0,9}` |
|          | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                          | Regex `[^\p{C}]{1,100}`           |
|          | Max        | Generate an arbitrary string.                                                                                                             | `String::arbitrary()`             |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`Parameter.yaml`](https://github.com/stencila/stencila/blob/main/schema/Parameter.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
