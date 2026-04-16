---
title: Execution Bounds
description: A boundary for node execution.
---

This is an enumeration used in Stencila Schema for limits placed on node execution.

It exists to let Stencila describe how far execution should proceed in a
document or workflow context, using a stable controlled vocabulary instead of
ad hoc flags. This is useful for user interfaces, execution planning, and
reproducible automation.

See the execution control properties that reference this enumeration.


# Members

The `ExecutionBounds` type has these members:

| Member | Description                                                          |
| ------ | -------------------------------------------------------------------- |
| `Main` | Execute within the main set of kernels with full capabilities.       |
| `Fork` | Execute within a forked set of kernels with full capabilities.       |
| `Box`  | Execute within a forked set of kernels with restricted capabilities. |

# Bindings

The `ExecutionBounds` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionBounds.jsonld)
- [JSON Schema](https://stencila.org/ExecutionBounds.schema.json)
- Python type [`ExecutionBounds`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`ExecutionBounds`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_bounds.rs)
- TypeScript type [`ExecutionBounds`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionBounds.ts)

***

This documentation was generated from [`ExecutionBounds.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionBounds.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
