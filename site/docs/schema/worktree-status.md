---
title: Worktree Status
description: The status of a source worktree relative to a commit.
---

This enumeration is used with source metadata such as `repository`, `path`,
and `commit` on `CreativeWork` and derived types.

It records whether the source worktree had uncommitted or untracked changes
when the work metadata was captured. Omit the property when status was not
checked or does not apply.

This avoids overloading the `commit` property with sentinel values such as
"dirty" or "untracked" while still preserving the commit hash that the
worktree state is relative to.


# Members

The `WorktreeStatus` type has these members:

| Member      | Description                                                                                     |
| ----------- | ----------------------------------------------------------------------------------------------- |
| `Clean`     | The source worktree matched the recorded commit with no known uncommitted or untracked changes. |
| `Dirty`     | The source worktree had tracked changes that were not represented by the recorded commit.       |
| `Untracked` | The source file or relevant worktree content was not tracked by version control.                |

# Bindings

The `WorktreeStatus` type is represented in:

- [JSON-LD](https://stencila.org/WorktreeStatus.jsonld)
- [JSON Schema](https://stencila.org/WorktreeStatus.schema.json)
- Python type [`WorktreeStatus`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`WorktreeStatus`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/worktree_status.rs)
- TypeScript type [`WorktreeStatus`](https://github.com/stencila/stencila/blob/main/ts/src/types/WorktreeStatus.ts)

***

This documentation was generated from [`WorktreeStatus.yaml`](https://github.com/stencila/stencila/blob/main/schema/WorktreeStatus.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
