---
title: "`stencila workflows run`"
description: Run a workflow
---

Run a workflow

Executes a workflow pipeline. Discovers the workflow by name, parses the DOT pipeline, resolves agents, and runs the pipeline through the attractor engine. Currently uses stub backends that log what they would do.

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

# Dry run to see pipeline config
stencila workflows run code-review --dry-run
```

# Arguments

| Name     | Description                      |
| -------- | -------------------------------- |
| `<NAME>` | The name of the workflow to run. |

# Options

| Name                   | Description                                                                            |
| ---------------------- | -------------------------------------------------------------------------------------- |
| `-g, --goal`           | Override the pipeline goal.                                                            |
| `-v, --verbose`        | Show detailed output with prompts and responses. Possible values: `true`, `false`.     |
| `--dry-run`            | Show workflow config and pipeline without executing. Possible values: `true`, `false`. |
| `--auto-approve`       | Auto-approve all human gates immediately. Possible values: `true`, `false`.            |
| `--auto-approve-after` | Auto-approve human gates after a duration.                                             |
