---
title: Execution Status
description: The status of a node's most recent execution.
---

This is an enumeration used in Stencila Schema for the status of an execution.

It exists so the lifecycle of executable nodes can be represented uniformly
across different execution engines and document types. This gives tools a
stable vocabulary for pending, running, succeeded, failed, and related states.

See [`Executable.executionStatus`](./executable.md#executionstatus) for the
property that uses this enumeration.


# Members

The `ExecutionStatus` type has these members:

| Member        | Description                                                                              |
| ------------- | ---------------------------------------------------------------------------------------- |
| `Scheduled`   | Execution of the node has been scheduled for some time in the future.                    |
| `Pending`     | Execution of the node is pending.                                                        |
| `Skipped`     | Execution of the node or node type was explicitly skipped by the user.                   |
| `Locked`      | Execution of the node was skipped because it is locked.                                  |
| `Rejected`    | Execution of the node was skipped because it is a rejected suggestion.                   |
| `Empty`       | Execution of the node was skipped because it has code, or other property, that is empty. |
| `Running`     | The node is currently being executed.                                                    |
| `Succeeded`   | Execution of the node completed without warning, error, or exception messages.           |
| `Warnings`    | Execution of the node completed but with warning messages.                               |
| `Errors`      | Execution of the node completed but with error messages.                                 |
| `Exceptions`  | Execution of the node did not complete because there was an exception message.           |
| `Cancelled`   | Execution of the node was pending but was cancelled.                                     |
| `Interrupted` | Execution of the node was running but was interrupted.                                   |

# Bindings

The `ExecutionStatus` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionStatus.jsonld)
- [JSON Schema](https://stencila.org/ExecutionStatus.schema.json)
- Python type [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_status.rs)
- TypeScript type [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionStatus.ts)

***

This documentation was generated from [`ExecutionStatus.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionStatus.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
