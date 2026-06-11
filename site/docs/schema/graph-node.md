---
title: Graph Node
description: A node in a graph.
---

This is a graph node type used in Stencila Schema for representing provenance,
reactivity and other graphs.

The wrapper gives every graph node a required graph-local `id` for edge
endpoints and embeds the represented Stencila resource or document node.


# Properties

The `GraphNode` type has these properties:

| Name   | Description                                                                  | Type                    | Inherited from |
| ------ | ---------------------------------------------------------------------------- | ----------------------- | -------------- |
| `id`   | The durable graph-local id used by graph edges to reference this graph node. | [`String`](./string.md) | -              |
| `node` | The embedded Stencila node represented by this graph node.                   | [`Node`](./node.md)     | -              |

# Related

The `GraphNode` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `GraphNode` type is represented in:

- [JSON-LD](https://stencila.org/GraphNode.jsonld)
- [JSON Schema](https://stencila.org/GraphNode.schema.json)
- Python class [`GraphNode`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`GraphNode`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/graph_node.rs)
- TypeScript class [`GraphNode`](https://github.com/stencila/stencila/blob/main/ts/src/types/GraphNode.ts)

***

This documentation was generated from [`GraphNode.yaml`](https://github.com/stencila/stencila/blob/main/schema/GraphNode.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
