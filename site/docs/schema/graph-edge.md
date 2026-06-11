---
title: Graph Edge
description: A directed edge in a graph.
---

This is a graph edge type used in Stencila for representing provenance,
reactivity and other graphs.

Edge direction is always from `source` to `target`. For resource-flow edges,
this is usually from upstream dependency to downstream dependant. For
discourse edges, this is the authored relation direction. Both endpoints
reference `GraphNode.id` values.


# Properties

The `GraphEdge` type has these properties:

| Name       | Description                                        | Type                                    | Inherited from          |
| ---------- | -------------------------------------------------- | --------------------------------------- | ----------------------- |
| `source`   | The id of the source graph node.                   | [`String`](./string.md)                 | -                       |
| `target`   | The id of the target graph node.                   | [`String`](./string.md)                 | -                       |
| `kind`     | The kind of relationship represented by this edge. | [`GraphEdgeKind`](./graph-edge-kind.md) | -                       |
| `evidence` | Evidence supporting the edge.                      | [`GraphEvidence`](./graph-evidence.md)* | -                       |
| `actions`  | Concrete activities associated with the edge.      | [`GraphAction`](./graph-action.md)*     | -                       |
| `id`       | The identifier for this item.                      | [`String`](./string.md)                 | [`Entity`](./entity.md) |

# Related

The `GraphEdge` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `GraphEdge` type is represented in:

- [JSON-LD](https://stencila.org/GraphEdge.jsonld)
- [JSON Schema](https://stencila.org/GraphEdge.schema.json)
- Python class [`GraphEdge`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`GraphEdge`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/graph_edge.rs)
- TypeScript class [`GraphEdge`](https://github.com/stencila/stencila/blob/main/ts/src/types/GraphEdge.ts)

***

This documentation was generated from [`GraphEdge.yaml`](https://github.com/stencila/stencila/blob/main/schema/GraphEdge.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
