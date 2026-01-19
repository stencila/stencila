---
title: "`stencila outputs push`"
description: Push outputs to Stencila Cloud
---

Push outputs to Stencila Cloud

# Usage

```sh
stencila outputs push [OPTIONS] [OUTPUTS]...
```

# Examples

```bash
# Push all outputs
stencila outputs push

# Push specific outputs
stencila outputs push "report.pdf"

# Push outputs matching a pattern
stencila outputs push "*.pdf"

# Dry run to preview what would be uploaded
stencila outputs push --dry-run

# Force push, ignoring refs filter
stencila outputs push --force
```

# Arguments

| Name        | Description                              |
| ----------- | ---------------------------------------- |
| `[OUTPUTS]` | Specific outputs to push (all if empty). |

# Options

| Name                  | Description                                                                                      |
| --------------------- | ------------------------------------------------------------------------------------------------ |
| `-f, --force <FORCE>` | Force push (ignore refs filter and re-upload unchanged files). Possible values: `true`, `false`. |
| `--dry-run <DRY_RUN>` | Dry run - process but don't upload. Possible values: `true`, `false`.                            |
