# Execution Status

**Status of the most recent, including any current, execution of a document node.**

**`@id`**: `stencila:ExecutionStatus`

## Members

The `ExecutionStatus` type has these members:

- `Scheduled`
- `ScheduledPreviouslyFailed`
- `Running`
- `RunningPreviouslyFailed`
- `Succeeded`
- `Failed`
- `Cancelled`

## Bindings

The `ExecutionStatus` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/ExecutionStatus.jsonld)
- [JSON Schema](https://stencila.dev/ExecutionStatus.schema.json)
- Python type [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/python/stencila/types/execution_status.py)
- Rust type [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_status.rs)
- TypeScript type [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/typescript/src/types/ExecutionStatus.ts)

## Source

This documentation was generated from [`ExecutionStatus.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionStatus.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).