---
title: Execution Status
description: Status of the most recent, including any current, execution of a document node.
config:
  publish:
    ghost:
      type: page
      slug: execution-status
      state: publish
      tags:
      - '#schema'
      - '#doc'
      - Flow
---

## Members

The `ExecutionStatus` type has these members:

- `Scheduled`
- `Pending`
- `Skipped`
- `Locked`
- `Rejected`
- `Empty`
- `Running`
- `Succeeded`
- `Warnings`
- `Errors`
- `Exceptions`
- `Cancelled`
- `Interrupted`

## Bindings

The `ExecutionStatus` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionStatus.jsonld)
- [JSON Schema](https://stencila.org/ExecutionStatus.schema.json)
- Python type [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/execution_status.py)
- Rust type [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_status.rs)
- TypeScript type [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionStatus.ts)

## Source

This documentation was generated from [`ExecutionStatus.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionStatus.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
