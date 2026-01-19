---
title: "`stencila outputs`"
description: Manage workspace outputs
---

Manage workspace outputs

# Usage

```sh
stencila outputs [COMMAND]
```

# Examples

```bash
# List configured outputs
stencila outputs
stencila outputs list
stencila outputs list --as toml

# Add an output
stencila outputs add report.pdf report.md
stencila outputs add report.pdf report.md --command render --refs main

# Remove an output
stencila outputs remove report.pdf

# Push all outputs to cloud
stencila outputs push

# Push specific outputs
stencila outputs push "report.pdf" "data/*.csv"

# Dry run (process but don't upload)
stencila outputs push --dry-run

# Force push (ignore refs filter)
stencila outputs push --force
```

# Subcommands

| Command               | Description                    |
| --------------------- | ------------------------------ |
| [`list`](list.md)     | List configured outputs        |
| [`add`](add.md)       | Add an output configuration    |
| [`remove`](remove.md) | Remove an output configuration |
| [`push`](push.md)     | Push outputs to Stencila Cloud |
