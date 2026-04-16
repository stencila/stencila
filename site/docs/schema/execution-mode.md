---
title: Execution Mode
description: The circumstances under which a node should be executed.
---

This is an enumeration used in Stencila Schema controlling when executable nodes
should run.

It exists so execution policy can be represented consistently across code,
prompts, forms, and other executable nodes. Using a controlled vocabulary
allows tools and renderers to agree on execution behavior without
format-specific conventions.

See [`Executable.executionMode`](./executable.md#executionmode) for the main
property that uses this enumeration.


# Members

The `ExecutionMode` type has these members:

| Member   | Description                                                                                                                                                                                    |
| -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Demand` | Execute on demand only.                                                                                                                                                                        |
| `Need`   | Execute on demand and, if the node is stale, when the document or ancestor node is executed.                                                                                                   |
| `Always` | Execute on demand and whenever the document or ancestor node is executed. Use this for nodes that you want to always be executed, even if they, or their upstream dependencies, are not stale. |
| `Auto`   | Execute on demand, and automatically if it is stale, including if is an upstream dependency of a node that is to be executed, or is a downstream dependant of a node that has been executed.   |
| `Lock`   | Do not execute the node. Requires that the node is unlocked first to be executed.                                                                                                              |

# Bindings

The `ExecutionMode` type is represented in:

- [JSON-LD](https://stencila.org/ExecutionMode.jsonld)
- [JSON Schema](https://stencila.org/ExecutionMode.schema.json)
- Python type [`ExecutionMode`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`ExecutionMode`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_mode.rs)
- TypeScript type [`ExecutionMode`](https://github.com/stencila/stencila/blob/main/ts/src/types/ExecutionMode.ts)

***

This documentation was generated from [`ExecutionMode.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionMode.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
