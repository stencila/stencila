---
title: "`stencila workflows resume`"
description: Resume a failed, cancelled, or interrupted workflow run
---

Resume a failed, cancelled, or interrupted workflow run

Continues execution of a previously failed, cancelled, or interrupted workflow run from where it left off. The pipeline state (completed nodes, context values, edge traversal history) is restored from the workspace database, and execution resumes at the next unfinished node.

If no run ID is provided, the most recent resumable run (failed, cancelled, or still marked as running) is used.

# Usage

```sh
stencila workflows resume [OPTIONS] [RUN_ID]
```

# Examples

```bash
# Resume the last failed or interrupted run
stencila workflows resume

# Resume a specific run by ID
stencila workflows resume 01926f3a-7b2c-7d4e-8f1a-9c3d5e7f0a1b

# Resume with verbose output
stencila workflows resume --verbose

# List runs to find a run ID, then resume it
stencila workflows runs
stencila workflows resume 01926f3a
```

# Arguments

| Name       | Description           |
| ---------- | --------------------- |
| `[RUN_ID]` | The run ID to resume. |

# Options

| Name            | Description                                                                              |
| --------------- | ---------------------------------------------------------------------------------------- |
| `-v, --verbose` | Show detailed output with prompts and responses. Possible values: `true`, `false`.       |
| `--force`       | Force resume of a run that is still marked as running. Possible values: `true`, `false`. |
