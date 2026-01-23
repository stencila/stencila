---
title: If Block
description: Show and execute alternative content conditional upon an executed expression.
---

This type is marked as unstable and is subject to change.

# Properties

The `IfBlock` type has these properties:

| Name                    | Description                                                      | Type                                                | Inherited from                  |
| ----------------------- | ---------------------------------------------------------------- | --------------------------------------------------- | ------------------------------- |
| `id`                    | The identifier for this item.                                    | [`String`](./string.md)                             | [`Entity`](./entity.md)         |
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
| `clauses`               | The clauses making up the `IfBlock` node                         | [`IfBlockClause`](./if-block-clause.md)*            | -                               |

# Related

The `IfBlock` type is related to these types:

- Parents: [`Executable`](./executable.md)
- Children: none

# Bindings

The `IfBlock` type is represented in:

- [JSON-LD](https://stencila.org/IfBlock.jsonld)
- [JSON Schema](https://stencila.org/IfBlock.schema.json)
- Python class [`IfBlock`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`IfBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/if_block.rs)
- TypeScript class [`IfBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/IfBlock.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `IfBlock` type are generated using the following strategies.

::: table

| Property  | Complexity | Description                                    | Strategy                                                   |
| --------- | ---------- | ---------------------------------------------- | ---------------------------------------------------------- |
| `clauses` | Min+       | A single fixed clause with a single paragraph. | `vec![ibc("true", None::<String>, [p([t("If clause")])])]` |
|           | Low+       | Generate up to 3 arbitrary if clauses          | `vec(IfBlockClause::arbitrary(), size_range(1..=3))`       |
|           | High+      | Generate up to 5 arbitrary if clauses          | `vec(IfBlockClause::arbitrary(), size_range(1..=10))`      |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

***

This documentation was generated from [`IfBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/IfBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
