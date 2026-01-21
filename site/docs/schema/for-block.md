---
title: For Block
description: Repeat a block content for each item in an array.
---

This type is marked as unstable and is subject to change.

# Properties

The `ForBlock` type has these properties:

| Name                    | Description                                                                   | Type                                                | Inherited from                           |
| ----------------------- | ----------------------------------------------------------------------------- | --------------------------------------------------- | ---------------------------------------- |
| `id`                    | The identifier for this item.                                                 | [`String`](./string.md)                             | [`Entity`](./entity.md)                  |
| `executionMode`         | Under which circumstances the node should be executed.                        | [`ExecutionMode`](./execution-mode.md)              | [`Executable`](./executable.md)          |
| `compilationDigest`     | A digest of the content, semantics and dependencies of the node.              | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md)          |
| `compilationMessages`   | Messages generated while compiling the code.                                  | [`CompilationMessage`](./compilation-message.md)*   | [`Executable`](./executable.md)          |
| `executionDigest`       | The `compilationDigest` of the node when it was last executed.                | [`CompilationDigest`](./compilation-digest.md)      | [`Executable`](./executable.md)          |
| `executionDependencies` | The upstream dependencies of this node.                                       | [`ExecutionDependency`](./execution-dependency.md)* | [`Executable`](./executable.md)          |
| `executionDependants`   | The downstream dependants of this node.                                       | [`ExecutionDependant`](./execution-dependant.md)*   | [`Executable`](./executable.md)          |
| `executionTags`         | Tags in the code which affect its execution.                                  | [`ExecutionTag`](./execution-tag.md)*               | [`Executable`](./executable.md)          |
| `executionCount`        | A count of the number of times that the node has been executed.               | [`Integer`](./integer.md)                           | [`Executable`](./executable.md)          |
| `executionRequired`     | Whether, and why, the code requires execution or re-execution.                | [`ExecutionRequired`](./execution-required.md)      | [`Executable`](./executable.md)          |
| `executionStatus`       | Status of the most recent, including any current, execution.                  | [`ExecutionStatus`](./execution-status.md)          | [`Executable`](./executable.md)          |
| `executionInstance`     | The id of the kernel instance that performed the last execution.              | [`String`](./string.md)                             | [`Executable`](./executable.md)          |
| `executionEnded`        | The timestamp when the last execution ended.                                  | [`Timestamp`](./timestamp.md)                       | [`Executable`](./executable.md)          |
| `executionDuration`     | Duration of the last execution.                                               | [`Duration`](./duration.md)                         | [`Executable`](./executable.md)          |
| `executionMessages`     | Messages emitted while executing the node.                                    | [`ExecutionMessage`](./execution-message.md)*       | [`Executable`](./executable.md)          |
| `code`                  | The code.                                                                     | [`Cord`](./cord.md)                                 | [`CodeExecutable`](./code-executable.md) |
| `programmingLanguage`   | The programming language of the code.                                         | [`String`](./string.md)                             | [`CodeExecutable`](./code-executable.md) |
| `executionBounds`       | The environment in which code should be executed.                             | [`ExecutionBounds`](./execution-bounds.md)          | [`CodeExecutable`](./code-executable.md) |
| `executionBounded`      | The execution bounds, if any, on the last execution.                          | [`ExecutionBounds`](./execution-bounds.md)          | [`CodeExecutable`](./code-executable.md) |
| `authors`               | The authors of the executable code.                                           | [`Author`](./author.md)*                            | [`CodeExecutable`](./code-executable.md) |
| `provenance`            | A summary of the provenance of the code.                                      | [`ProvenanceCount`](./provenance-count.md)*         | [`CodeExecutable`](./code-executable.md) |
| `variable`              | The name to give to the variable representing each item in the iterated array | [`String`](./string.md)                             | -                                        |
| `content`               | The content to repeat for each item                                           | [`Block`](./block.md)*                              | -                                        |
| `otherwise`             | The content to render if there are no items                                   | [`Block`](./block.md)*                              | -                                        |
| `iterations`            | The content repeated for each iteration                                       | [`Block`](./block.md)*                              | -                                        |

# Related

The `ForBlock` type is related to these types:

- Parents: [`CodeExecutable`](./code-executable.md)
- Children: none

# Bindings

The `ForBlock` type is represented in:

- [JSON-LD](https://stencila.org/ForBlock.jsonld)
- [JSON Schema](https://stencila.org/ForBlock.schema.json)
- Python class [`ForBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/for_block.py)
- Rust struct [`ForBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/for_block.rs)
- TypeScript class [`ForBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/ForBlock.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `ForBlock` type are generated using the following strategies.

::: table

| Property              | Complexity | Description                                                                                                                               | Strategy                                      |
| --------------------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- |
| `code`                | Min+       | Generate a simple fixed string of code.                                                                                                   | `Cord::from("code")`                          |
|                       | Low+       | Generate a random string of up to 10 alphanumeric characters (excludes whitespace which<br><br>can be problematic in Markdown).           | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)`   |
|                       | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                          | `r"[^\p{C}]{1,100}".prop_map(Cord::from)`     |
|                       | Max        | Generate an arbitrary string.                                                                                                             | `String::arbitrary().prop_map(Cord::from)`    |
| `programmingLanguage` | Min+       | Generate a simple fixed string.                                                                                                           | `Some(String::from("lang"))`                  |
|                       | Low+       | Generate one of the well known programming language short names.                                                                          | `option::of(r"(cpp)\|(js)\|(py)\|(r)\|(ts)")` |
|                       | High+      | Generate a random string of up to 10 alphanumeric characters.                                                                             | `option::of(r"[a-zA-Z0-9]{1,10}")`            |
|                       | Max        | Generate an arbitrary string.                                                                                                             | `option::of(String::arbitrary())`             |
| `variable`            | Min+       | Generate a fixed variable name.                                                                                                           | `String::from("item")`                        |
|                       | Low+       | Generate a random string of up to 10 alphanumeric characters (and at most one underscore to avoid<br><br>a clash with Markdown emphasis). | Regex `[a-zA-Z_][a-zA-Z0-9]{0,9}`             |
|                       | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                          | Regex `[^\p{C}]{1,100}`                       |
|                       | Max        | Generate an arbitrary string.                                                                                                             | `String::arbitrary()`                         |
| `content`             | Min+       | A single simple paragraph.                                                                                                                | `vec![p([t("For content")])]`                 |
|                       | Low+       | Generate up to four arbitrary, non-recursive, block nodes.                                                                                | `vec_blocks_non_recursive(4)`                 |
|                       | Max        | Generate up to eight arbitrary, non-recursive, block nodes.                                                                               | `vec_blocks_non_recursive(8)`                 |
| `otherwise`           | Min+       | No otherwise clause.                                                                                                                      | `None`                                        |
|                       | Low+       | Generate up to two arbitrary, non-recursive, block nodes.                                                                                 | `option::of(vec_blocks_non_recursive(2))`     |
|                       | Max        | Generate up to four arbitrary, non-recursive, block nodes.                                                                                | `option::of(vec_blocks_non_recursive(4))`     |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`ForBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/ForBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
