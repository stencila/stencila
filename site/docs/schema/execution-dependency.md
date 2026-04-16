---
title: Execution Dependency
description: An upstream execution dependency of a node.
---

This is a type used in Stencila Schema for describing an upstream execution
dependency of a node.

It exists to make the inputs and prerequisites of executable nodes explicit in
the schema, enabling incremental execution, invalidation, and inspection of
execution graphs.

Key properties identify the dependency node and the relation that explains why
it is required.


# Analogues

The following external types, elements, or nodes are similar to a `ExecutionDependency`:

- [build dependency edge](https://en.wikipedia.org/wiki/Dependency_graph): Close graph-theoretic analogue for an upstream dependency relation between executable units, though Stencila also records node identity and code location metadata.

# Properties

The `ExecutionDependency` type has these properties:

| Name                 | Description                                  | Type                                                                | Inherited from          |
| -------------------- | -------------------------------------------- | ------------------------------------------------------------------- | ----------------------- |
| `dependencyRelation` | The relation to the dependency.              | [`ExecutionDependencyRelation`](./execution-dependency-relation.md) | -                       |
| `dependencyType`     | The type of node that is the dependency.     | [`String`](./string.md)                                             | -                       |
| `dependencyId`       | The id of node that is the dependency.       | [`String`](./string.md)                                             | -                       |
| `codeLocation`       | The location that the dependency is defined. | [`CodeLocation`](./code-location.md)                                | -                       |
| `id`                 | The identifier for this item.                | [`String`](./string.md)                                             | [`Entity`](./entity.md) |

# Related

The `ExecutionDependency` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ExecutionDependency` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionDependency.jsonld)
- [JSON Schema](https://stencila.org/ExecutionDependency.schema.json)
- Python class [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_dependency.rs)
- TypeScript class [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionDependency.ts)

***

This documentation was generated from [`ExecutionDependency.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionDependency.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
