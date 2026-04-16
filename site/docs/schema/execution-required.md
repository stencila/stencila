---
title: Execution Required
description: A reason why a node does or does not require execution.
---

This is an enumeration used in Stencila Schema describing whether and why execution
is required.

It exists to distinguish different re-execution states, such as stale
dependencies or changed content, in a way that user interfaces and automation
can interpret consistently.

See [`Executable.executionRequired`](./executable.md#executionrequired) for
the property that uses this enumeration.


# Members

The `ExecutionRequired` type has these members:

| Member                 | Description                                                                                                                                                                             |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `No`                   | No re-execution is required, the semantics of the node and its dependencies has not changed since it was last executed                                                                  |
| `NeverExecuted`        | Execution is required because the node has never been executed (or any previous execution was not persisted in its state).                                                              |
| `StateChanged`         | Re-execution is required because the state of the node (e.g. source code) has changed since it was last executed and no semantic digest is available to determine if semantics changed. |
| `SemanticsChanged`     | Re-execution is required because the semantics of the node has changed since it was last executed.                                                                                      |
| `DependenciesChanged`  | Re-execution is required because the semantics of one or more dependencies (including transitive dependencies) changed since it was last executed.                                      |
| `DependenciesFailed`   | Re-execution is required because one or more dependencies (including transitive dependencies) failed when it was last executed.                                                         |
| `ExecutionFailed`      | Re-execution is required because execution failed (there were errors or exceptions) the last time it was executed.                                                                      |
| `ExecutionCancelled`   | Re-execution may be required because execution was pending but was cancelled.                                                                                                           |
| `ExecutionInterrupted` | Re-execution is required because execution was interrupted the last time it was executed.                                                                                               |
| `KernelRestarted`      | Re-execution is required because the kernel that the node was last executed in was restarted.                                                                                           |
| `UserRequested`        | Execution is required because it was explicitly requested by a user.                                                                                                                    |

# Bindings

The `ExecutionRequired` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionRequired.jsonld)
- [JSON Schema](https://stencila.org/ExecutionRequired.schema.json)
- Python type [`ExecutionRequired`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`ExecutionRequired`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_required.rs)
- TypeScript type [`ExecutionRequired`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionRequired.ts)

***

This documentation was generated from [`ExecutionRequired.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionRequired.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
