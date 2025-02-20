---
title: Execution Mode
description: Under which circumstances a node should be executed.
config:
  publish:
    ghost:
      type: page
      slug: execution-mode
      state: publish
      tags:
      - '#schema'
      - '#doc'
      - Flow
---

## Members

The `ExecutionMode` type has these members:

- `Demand`
- `Need`
- `Always`
- `Auto`
- `Lock`

## Bindings

The `ExecutionMode` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionMode.jsonld)
- [JSON Schema](https://stencila.org/ExecutionMode.schema.json)
- Python type [`ExecutionMode`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/execution_mode.py)
- Rust type [`ExecutionMode`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_mode.rs)
- TypeScript type [`ExecutionMode`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionMode.ts)

## Source

This documentation was generated from [`ExecutionMode.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionMode.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
