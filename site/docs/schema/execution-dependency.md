---
title: Execution Dependency
description: An upstream execution dependency of a node.
---

# Properties

The `ExecutionDependency` type has these properties:

| Name                 | Description                                  | Type                                                                | Inherited from          |
| -------------------- | -------------------------------------------- | ------------------------------------------------------------------- | ----------------------- |
| `id`                 | The identifier for this item.                | [`String`](./string.md)                                             | [`Entity`](./entity.md) |
| `dependencyRelation` | The relation to the dependency.              | [`ExecutionDependencyRelation`](./execution-dependency-relation.md) | -                       |
| `dependencyType`     | The type of node that is the dependency.     | [`String`](./string.md)                                             | -                       |
| `dependencyId`       | The id of node that is the dependency.       | [`String`](./string.md)                                             | -                       |
| `codeLocation`       | The location that the dependency is defined. | [`CodeLocation`](./code-location.md)                                | -                       |

# Related

The `ExecutionDependency` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ExecutionDependency` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionDependency.jsonld)
- [JSON Schema](https://stencila.org/ExecutionDependency.schema.json)
- Python class [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/execution_dependency.py)
- Rust struct [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_dependency.rs)
- TypeScript class [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionDependency.ts)

# Source

This documentation was generated from [`ExecutionDependency.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionDependency.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
