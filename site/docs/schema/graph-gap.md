---
title: Graph Gap
description: A sourced limitation of an account, occurrence, relationship or evidentiary basis.
---

Gaps qualify the account of their source, not evidence from other sources. An
absent gaps list means no gaps were reported, never that coverage is complete.
A gap may stand for several like items; its count is not an item count.
Resource identities must survive graph-local node renumbering. Withheld gaps
omit any identity whose disclosure would reveal the withheld material.


# Properties

The `GraphGap` type has these properties:

| Name          | Description                                                            | Type                                                            | Inherited from          |
| ------------- | ---------------------------------------------------------------------- | --------------------------------------------------------------- | ----------------------- |
| `state`       | The distinct availability or coverage state.                           | [`GraphGapState`](./graph-gap-state.md)                         | -                       |
| `source`      | The producer whose account this gap qualifies.                         | [`ThingVariant`](./thing-variant.md) \| [`String`](./string.md) | -                       |
| `target`      | The structured scope of the limitation.                                | [`GraphGapTarget`](./graph-gap-target.md)                       | -                       |
| `resource`    | The affected resource's durable identity, never a graph-local node id. | [`String`](./string.md)                                         | -                       |
| `code`        | A stable producer-specific reason code.                                | [`String`](./string.md)                                         | -                       |
| `description` | Human-readable scope of the limitation.                                | [`String`](./string.md)                                         | -                       |
| `id`          | The identifier for this item.                                          | [`String`](./string.md)                                         | [`Entity`](./entity.md) |

# Related

The `GraphGap` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `GraphGap` type is represented in:

- [JSON-LD](https://stencila.org/GraphGap.jsonld)
- [JSON Schema](https://stencila.org/GraphGap.schema.json)
- Python class [`GraphGap`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`GraphGap`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/graph_gap.rs)
- TypeScript class [`GraphGap`](https://github.com/stencila/stencila/blob/main/ts/src/types/GraphGap.ts)

***

This documentation was generated from [`GraphGap.yaml`](https://github.com/stencila/stencila/blob/main/schema/GraphGap.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
