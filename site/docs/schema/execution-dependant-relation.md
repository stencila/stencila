---
title: Execution Dependant Relation
description: A downstream execution relation between nodes.
---

This is an enumeration used in Stencila Schema describing how one node is a
downstream dependant of another.

It exists to classify dependency-graph edges with semantics that are
meaningful for execution planning and invalidation, rather than treating all
downstream links as equivalent.

See
[`ExecutionDependant.dependantRelation`](./execution-dependant.md#dependantrelation)
for the property that uses this enumeration.


# Members

The `ExecutionDependantRelation` type has these members:

| Member     | Description                                             |
| ---------- | ------------------------------------------------------- |
| `Assigns`  | The node assigns its dependant (usually a variable)     |
| `Alters`   | The node alters its dependant (usually a variable)      |
| `Declares` | The node declares its dependant (e.g. a database table) |
| `Writes`   | The node writes its dependant (usually a file)          |

# Bindings

The `ExecutionDependantRelation` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionDependantRelation.jsonld)
- [JSON Schema](https://stencila.org/ExecutionDependantRelation.schema.json)
- Python type [`ExecutionDependantRelation`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`ExecutionDependantRelation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_dependant_relation.rs)
- TypeScript type [`ExecutionDependantRelation`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionDependantRelation.ts)

***

This documentation was generated from [`ExecutionDependantRelation.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionDependantRelation.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
