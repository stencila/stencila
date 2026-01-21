---
title: Execution Dependant
description: A downstream execution dependant of a node.
---

# Properties

The `ExecutionDependant` type has these properties:

| Name                | Description                                 | Type                                                              | Inherited from          |
| ------------------- | ------------------------------------------- | ----------------------------------------------------------------- | ----------------------- |
| `id`                | The identifier for this item.               | [`String`](./string.md)                                           | [`Entity`](./entity.md) |
| `dependantRelation` | The relation to the dependant.              | [`ExecutionDependantRelation`](./execution-dependant-relation.md) | -                       |
| `dependantType`     | The type of node that is the dependant.     | [`String`](./string.md)                                           | -                       |
| `dependantId`       | The id of node that is the dependant.       | [`String`](./string.md)                                           | -                       |
| `codeLocation`      | The location that the dependant is defined. | [`CodeLocation`](./code-location.md)                              | -                       |

# Related

The `ExecutionDependant` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `ExecutionDependant` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionDependant.jsonld)
- [JSON Schema](https://stencila.org/ExecutionDependant.schema.json)
- Python class [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/execution_dependant.py)
- Rust struct [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_dependant.rs)
- TypeScript class [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionDependant.ts)

# Source

This documentation was generated from [`ExecutionDependant.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionDependant.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
