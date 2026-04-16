---
title: Execution Dependency Relation
description: An upstream execution relation between nodes.
---

This is an enumeration used in Stencila Schema describing how one node depends on
another for execution.

It exists to classify execution dependencies with more precision than a simple
linked list of prerequisites, helping Stencila reason about compilation,
execution order, and re-execution.

See
[`ExecutionDependency.dependencyRelation`](./execution-dependency.md#dependencyrelation)
for the property that uses this enumeration.


# Members

The `ExecutionDependencyRelation` type has these members:

| Member     | Description                                                          |
| ---------- | -------------------------------------------------------------------- |
| `Calls`    | The node calls its dependency (usually another document or function) |
| `Derives`  | The node is derived from its dependency (e.g. a database table)      |
| `Imports`  | The node imports its dependency (usually a software module)          |
| `Includes` | The node includes its dependency (usually another document)          |
| `Reads`    | The node reads its dependency (usually a file)                       |
| `Uses`     | The node uses its dependency (usually a variable)                    |

# Bindings

The `ExecutionDependencyRelation` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionDependencyRelation.jsonld)
- [JSON Schema](https://stencila.org/ExecutionDependencyRelation.schema.json)
- Python type [`ExecutionDependencyRelation`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`ExecutionDependencyRelation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_dependency_relation.rs)
- TypeScript type [`ExecutionDependencyRelation`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionDependencyRelation.ts)

***

This documentation was generated from [`ExecutionDependencyRelation.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionDependencyRelation.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
