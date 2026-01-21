---
title: Include Block
description: Include block content from an external source (e.g. file, URL).
---

This type is marked as unstable and is subject to change.

# Properties

The `IncludeBlock` type has these properties:

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
| `source`                | The external source of the content, a file path or URL.          | [`String`](./string.md)                             | -                               |
| `mediaType`             | Media type of the source content.                                | [`String`](./string.md)                             | -                               |
| `select`                | A query to select a subset of content from the source            | [`String`](./string.md)                             | -                               |
| `content`               | The structured content decoded from the source.                  | [`Block`](./block.md)*                              | -                               |

# Related

The `IncludeBlock` type is related to these types:

- Parents: [`Executable`](./executable.md)
- Children: [`CallBlock`](./call-block.md)

# Bindings

The `IncludeBlock` type is represented in:

- [JSON-LD](https://stencila.org/IncludeBlock.jsonld)
- [JSON Schema](https://stencila.org/IncludeBlock.schema.json)
- Python class [`IncludeBlock`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/include_block.py)
- Rust struct [`IncludeBlock`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/include_block.rs)
- TypeScript class [`IncludeBlock`](https://github.com/stencila/stencila/blob/main/ts/src/types/IncludeBlock.ts)

# Testing

During property-based (a.k.a generative) testing, the properties of the `IncludeBlock` type are generated using the following strategies.

::: table

| Property | Complexity | Description                                                                                                                                                                                                        | Strategy                              |
| -------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------- |
| `source` | Min+       | Generate a fixed path.                                                                                                                                                                                             | `String::from("path/to/source.file")` |
|          | Low+       | Generate a random string with up to 30 alphanumeric characters, forward slashes,<br><br>hyphens, and dots (exclude characters in such as underscores an asterisks which<br><br>have semantic meaning in Markdown). | Regex `[a-zA-Z0-9/\-.]{1,30}`         |
|          | High+      | Generate a random string of up to 100 characters (excluding control characters).                                                                                                                                   | Regex `[^\p{C}]{1,100}`               |
|          | Max        | Generate an arbitrary string.                                                                                                                                                                                      | `String::arbitrary()`                 |

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the[`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on proptest generation strategies listed.

:::

# Source

This documentation was generated from [`IncludeBlock.yaml`](https://github.com/stencila/stencila/blob/main/schema/IncludeBlock.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
