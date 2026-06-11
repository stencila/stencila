---
title: Action Status Type
description: The status of an action.
---

This is an implementation of schema.org
[`ActionStatusType`](https://schema.org/ActionStatusType).

It records whether an action is potential, active, completed, or failed. In
provenance graphs it allows imported, observed, and Stencila-native actions to
carry an explicit lifecycle state without relying on action-specific status
fields.


# Members

The `ActionStatusType` type has these members:

| Member                  | Description                                             |
| ----------------------- | ------------------------------------------------------- |
| `PotentialActionStatus` | The action is proposed or possible but has not started. |
| `ActiveActionStatus`    | The action is currently in progress.                    |
| `CompletedActionStatus` | The action has completed successfully.                  |
| `FailedActionStatus`    | The action failed.                                      |

# Bindings

The `ActionStatusType` type is represented in:

- [JSON-LD](https://stencila.org/ActionStatusType.jsonld)
- [JSON Schema](https://stencila.org/ActionStatusType.schema.json)
- Python type [`ActionStatusType`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`ActionStatusType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/action_status_type.rs)
- TypeScript type [`ActionStatusType`](https://github.com/stencila/stencila/blob/main/ts/src/types/ActionStatusType.ts)

***

This documentation was generated from [`ActionStatusType.yaml`](https://github.com/stencila/stencila/blob/main/schema/ActionStatusType.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
