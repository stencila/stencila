---
title: Graph Gap Target
description: The structured scope of a graph limitation.
---

Item qualifies the containing node, edge, evidence item or whole graph.
Property requires a property path; Relationship requires an edge kind and a
direction, and may name a role. Unused fields should be absent.


# Properties

The `GraphGapTarget` type has these properties:

| Name           | Description                                                                             | Type                                               | Inherited from          |
| -------------- | --------------------------------------------------------------------------------------- | -------------------------------------------------- | ----------------------- |
| `kind`         | Whether the limitation qualifies an item, property or relationship.                     | [`GraphGapTargetKind`](./graph-gap-target-kind.md) | -                       |
| `property`     | A property path, required for a Property target.                                        | [`String`](./string.md)                            | -                       |
| `relationship` | The edge kind, required for a Relationship target.                                      | [`GraphEdgeKind`](./graph-edge-kind.md)            | -                       |
| `direction`    | The edge direction relative to the containing item, required for a Relationship target. | [`GraphGapDirection`](./graph-gap-direction.md)    | -                       |
| `role`         | An optional relationship role narrowing its scope.                                      | [`String`](./string.md)                            | -                       |
| `id`           | The identifier for this item.                                                           | [`String`](./string.md)                            | [`Entity`](./entity.md) |

# Related

The `GraphGapTarget` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `GraphGapTarget` type is represented in:

- [JSON-LD](https://stencila.org/GraphGapTarget.jsonld)
- [JSON Schema](https://stencila.org/GraphGapTarget.schema.json)
- Python class [`GraphGapTarget`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`GraphGapTarget`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/graph_gap_target.rs)
- TypeScript class [`GraphGapTarget`](https://github.com/stencila/stencila/blob/main/ts/src/types/GraphGapTarget.ts)

***

This documentation was generated from [`GraphGapTarget.yaml`](https://github.com/stencila/stencila/blob/main/schema/GraphGapTarget.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
