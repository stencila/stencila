---
title: If Block Clause
description: A clause within an `IfBlock` node.
---

This type is marked as unstable and is subject to change.

# Properties

The `IfBlockClause` type has these properties:

| Name                    | Description                                                           | Type                                                | Inherited from                           |
| ----------------------- | --------------------------------------------------------------------- | --------------------------------------------------- | ---------------------------------------- |
| `id`                    | The identifier for this item.                                         | [`String`](./string.md)                             | [`Entity`](./entity.md)                  |
| `executionMode`         | Under which circumstances the node should be executed.                | [`ExecutionMode`](./execution-mode.md)              | [`Executable`](./executable.md)          |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.      | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md)          |
| `compilationMessages`   | Messages generated while compiling the code.                          | [`CompilationMessage`](./compilation-message.md)*   | [`Executable`](./executable.md)          |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.        | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md)          |
| `executionDependencies` | The upstream dependencies of this node.                               | [`ExecutionDependency`](./execution-dependency.md)* | [`Executable`](./executable.md)          |
| `executionDependants`   | The downstream dependants of this node.                               | [`ExecutionDependant`](./execution-dependant.md)*   | [`Executable`](./executable.md)          |
| `executionTags`         | Tags in the code which affect its execution.                          | [`ExecutionTag`](./execution-tag.md)*               | [`Executable`](./executable.md)          |
| `executionCount`        | A count of the number of times that the node has been executed.       | [`Integer`](./integer.md)                           | [`Executable`](./executable.md)          |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.        | [`ExecutionRequired`](./execution-required.md)      | [`Executable`](./executable.md)          |
| `executionStatus`       | Status of the most recent, including any current, execution.          | [`ExecutionStatus`](./execution-status.md)          | [`Executable`](./executable.md)          |
| `executionInstance`     | The id of the kernel instance that performed the last execution.      | [`String`](./string.md)                             | [`Executable`](./executable.md)          |
| `executionEnded`        | The timestamp when the last execution ended.                          | [`Timestamp`](./timestamp.md)                       | [`Executable`](./executable.md)          |
| `executionDuration`     | Duration of the last execution.                                       | [`Duration`](./duration.md)                         | [`Executable`](./executable.md)          |
| `executionMessages`     | Messages emitted while executing the node.                            | [`ExecutionMessage`](./execution-message.md)*       | [`Executable`](./executable.md)          |
| `code`                  | The code.                                                             | [`Cord`](./cord.md)                                 | [`CodeExecutable`](./code-executable.md) |
| `programmingLanguage`   | The programming language of the code.                                 | [`String`](./string.md)                             | [`CodeExecutable`](./code-executable.md) |
| `executionBounds`       | The environment in which code should be executed.                     | [`ExecutionBounds`](./execution-bounds.md)          | [`CodeExecutable`](./code-executable.md) |
| `executionBounded`      | The execution bounds, if any, on the last execution.                  | [`ExecutionBounds`](./execution-bounds.md)          | [`CodeExecutable`](./code-executable.md) |
| `authors`               | The authors of the executable code.                                   | [`Author`](./author.md)*                            | [`CodeExecutable`](./code-executable.md) |
| `provenance`            | A summary of the provenance of the code.                              | [`ProvenanceCount`](./provenance-count.md)*         | [`CodeExecutable`](./code-executable.md) |
| `isActive`              | Whether this clause is the active clause in the parent `IfBlock` node | [`Boolean`](./boolean.md)                           | -                                        |
| `content`               | The content to render if the result is truthy                         | [`Block`](./block.md)*                              | -                                        |

# Related

The `IfBlockClause` type is related to these types:

- Parents: [`CodeExecutable`](./code-executable.md)
- Children: none

# Bindings

The `IfBlockClause` type is represented in:

- [JSON-LD](https://stencila.org/IfBlockClause.jsonld)
- [JSON Schema](https://stencila.org/IfBlockClause.schema.json)
- Python class [`IfBlockClause`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/if_block_clause.py)
- Rust struct [`IfBlockClause`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/if_block_clause.rs)
- TypeScript class [`IfBlockClause`](https://github.com/stencila/stencila/blob/main/ts/src/types/IfBlockClause.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `IfBlockClause` type are generated using the following strategies.

::: table

| Property              | Complexity | Description                                                                                                                     | Strategy                                      |
| --------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- |
| `code`                | Min+       | Generate a simple fixed string of code.                                                                                         | `Cord::from("code")`                          |
|                       | Low+       | Generate a random string of up to 10 alphanumeric characters (excludes whitespace which<br><br>can be problematic in Markdown). | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)`   |
|                       | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`     |
|                       | Max        | Generate an arbitrary string.                                                                                                   | `String::arbitrary().prop_map(Cord::from)`    |
| `programmingLanguage` | Min+       | Generate a simple fixed string.                                                                                                 | `Some(String::from("lang"))`                  |
|                       | Low+       | Generate one of the well known programming language short names.                                                                | `option::of(r"(cpp)\|(js)\|(py)\|(r)\|(ts)")` |
|                       | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                   | `option::of(r"[a-zA-Z0-9]{1,10}")`            |
|                       | Max        | Generate an arbitrary string.                                                                                                   | `option::of(String::arbitrary())`             |
| `content`             | Min+       | A single, simple paragraph.                                                                                                     | `vec![p([t("If clause content")])]`           |
|                       | Low+       | Generate up to two arbitrary, non-recursive, block nodes.                                                                       | `vec_blocks_non_recursive(2)`                 |
|                       | Max        | Generate up to four arbitrary, non-recursive, block nodes.                                                                      | `vec_blocks_non_recursive(4)`                 |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`IfBlockClause.yaml`](https://github.com/stencila/stencila/blob/main/schema/IfBlockClause.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
