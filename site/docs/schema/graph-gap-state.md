---
title: Graph Gap State
description: Distinct evidence availability and coverage limitations.
---

# Members

The `GraphGapState` type has these members:

| Member       | Description                                                                  |
| ------------ | ---------------------------------------------------------------------------- |
| `Missing`    | The evidence or property is unavailable or was never recorded.               |
| `Expired`    | The material was removed by ordinary retention.                              |
| `Withheld`   | Authority or disclosure policy prevents returning the material.              |
| `Unobserved` | This source did not observe the operation or relationship.                   |
| `Purged`     | The owner explicitly removed the retained material.                          |
| `Omitted`    | Inspection or projection stopped at a budget, without claiming availability. |

# Bindings

The `GraphGapState` type is represented in:

- [JSON-LD](https://stencila.org/GraphGapState.jsonld)
- [JSON Schema](https://stencila.org/GraphGapState.schema.json)
- Python type [`GraphGapState`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`GraphGapState`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/graph_gap_state.rs)
- TypeScript type [`GraphGapState`](https://github.com/stencila/stencila/blob/main/ts/src/types/GraphGapState.ts)

***

This documentation was generated from [`GraphGapState.yaml`](https://github.com/stencila/stencila/blob/main/schema/GraphGapState.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
