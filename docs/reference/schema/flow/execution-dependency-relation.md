---
title: Execution Dependency Relation
description: The relation between a node and its execution dependency.
config:
  publish:
    ghost:
      type: page
      slug: execution-dependency-relation
      state: publish
      tags:
      - '#schema'
      - '#doc'
      - Flow
---

## Members

The `ExecutionDependencyRelation` type has these members:

- `Calls`
- `Derives`
- `Imports`
- `Includes`
- `Reads`
- `Uses`

## Bindings

The `ExecutionDependencyRelation` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionDependencyRelation.jsonld)
- [JSON Schema](https://stencila.org/ExecutionDependencyRelation.schema.json)
- Python type [`ExecutionDependencyRelation`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/execution_dependency_relation.py)
- Rust type [`ExecutionDependencyRelation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_dependency_relation.rs)
- TypeScript type [`ExecutionDependencyRelation`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionDependencyRelation.ts)

## Source

This documentation was generated from [`ExecutionDependencyRelation.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionDependencyRelation.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
