---
title: "`stencila workflows run`"
description: Run a workflow
---

Run a workflow

Validates and executes a workflow from the current workspace.

The command finds the workflow by name, applies any CLI overrides such as `--goal`, validates the definition, and then runs the pipeline using the configured workflow, shell, and agent handlers. By default it shows compact progress output; use `--verbose` to show prompts and responses for each stage, or `--dry-run` to preview the workflow configuration without executing it. Human approval gates wait by default, and can be auto-approved with `--auto-approve` or `--auto-approve-after`.

# Usage

```sh
stencila workflows run [OPTIONS] <NAME>
```

# Examples

```bash
# Run a workflow
stencila workflows run code-review

# Run with a goal override
stencila workflows run code-review --goal "Implement login feature"

# Preview workflow details and pipeline without executing
stencila workflows run code-review --dry-run

# Show detailed prompts and responses for each stage
stencila workflows run code-review --verbose

# Automatically approve human gates for unattended runs
stencila workflows run code-review --auto-approve

# Auto-approve human gates after 30 seconds
stencila workflows run code-review --auto-approve-after 30s
```

# Arguments

| Name     | Description                      |
| -------- | -------------------------------- |
| `<NAME>` | The name of the workflow to run. |

# Options

| Name                   | Description                                                                                |
| ---------------------- | ------------------------------------------------------------------------------------------ |
| `-g, --goal`           | Override the pipeline goal.                                                                |
| `-v, --verbose`        | Show detailed output with prompts and responses. Possible values: `true`, `false`.         |
| `--dry-run`            | Preview workflow details and pipeline without executing. Possible values: `true`, `false`. |
| `--auto-approve`       | Auto-approve all human gates immediately. Possible values: `true`, `false`.                |
| `--auto-approve-after` | Auto-approve human gates after a duration.                                                 |
