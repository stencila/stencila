---
title: Bibliography
description: A bibliography loaded from an external source file.
---

A `Bibliography` represents a database of references that may be cited in a
`CreativeWork`. Unlike the `references` property on `CreativeWork` (which contains
only the references that are actually cited), a `Bibliography` contains all available
references loaded from an external source file (e.g. BibTeX, YAML) at compile time.

The loaded references are ephemeral and are stripped when the document is saved,
to be reloaded on the next compilation.


This type is marked as unstable and is subject to change.

# Properties

The `Bibliography` type has these properties:

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
| `source`                | The external source of the bibliography, a file path or URL.     | [`String`](./string.md)                             | -                               |
| `mediaType`             | Media type of the source content.                                | [`String`](./string.md)                             | -                               |
| `references`            | The references loaded from the source.                           | [`Reference`](./reference.md)*                      | -                               |

# Related

The `Bibliography` type is related to these types:

- Parents: [`Executable`](./executable.md)
- Children: none

# Bindings

The `Bibliography` type is represented in:

- [JSON-LD](https://stencila.org/Bibliography.jsonld)
- [JSON Schema](https://stencila.org/Bibliography.schema.json)
- Python class [`Bibliography`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Bibliography`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/bibliography.rs)
- TypeScript class [`Bibliography`](https://github.com/stencila/stencila/blob/main/ts/src/types/Bibliography.ts)

***

This documentation was generated from [`Bibliography.yaml`](https://github.com/stencila/stencila/blob/main/schema/Bibliography.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
