---
title: Graph Action
description: An action associated with a graph edge.
---

This is a union type used by `GraphEdge.actions` for concrete activities that
explain how a resource-flow relationship came about.

Use graph edge actions only for performed, recorded, imported, or attested
operations. Static analysis, inferred relationships, and direct observations
of existing state should be recorded as `GraphEvidence` rather than as graph
actions.


# Members

The `GraphAction` type has these members:

- [`Action`](./action.md)
- [`CreateAction`](./create-action.md)
- [`ConvertAction`](./convert-action.md)
- [`ExecuteAction`](./execute-action.md)

# Bindings

The `GraphAction` type is represented in:

- [JSON-LD](https://stencila.org/GraphAction.jsonld)
- [JSON Schema](https://stencila.org/GraphAction.schema.json)
- Python type [`GraphAction`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`GraphAction`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/graph_action.rs)
- TypeScript type [`GraphAction`](https://github.com/stencila/stencila/blob/main/ts/src/types/GraphAction.ts)

***

This documentation was generated from [`GraphAction.yaml`](https://github.com/stencila/stencila/blob/main/schema/GraphAction.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
