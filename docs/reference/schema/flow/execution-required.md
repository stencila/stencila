---
title: Execution Required
description: Whether, and why, the execution of a node is required or not.
config:
  publish:
    ghost:
      type: post
      slug: execution-required
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Flow
---

# Members

The `ExecutionRequired` type has these members:

- `No`
- `NeverExecuted`
- `StateChanged`
- `SemanticsChanged`
- `DependenciesChanged`
- `DependenciesFailed`
- `ExecutionFailed`
- `ExecutionCancelled`
- `ExecutionInterrupted`
- `KernelRestarted`
- `UserRequested`

# Bindings

The `ExecutionRequired` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionRequired.jsonld)
- [JSON Schema](https://stencila.org/ExecutionRequired.schema.json)
- Python type [`ExecutionRequired`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/execution_required.py)
- Rust type [`ExecutionRequired`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_required.rs)
- TypeScript type [`ExecutionRequired`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionRequired.ts)

# Source

This documentation was generated from [`ExecutionRequired.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionRequired.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
