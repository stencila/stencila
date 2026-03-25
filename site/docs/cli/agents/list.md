---
title: "`stencila agents list`"
description: List available agents
---

List available agents

Shows agents from workspace `.stencila/agents/`, user config `~/.config/stencila/agents/`, and auto-detected CLI tools on PATH. Use `--source` to filter by source.

# Usage

```sh
stencila agents list [OPTIONS]
```

# Examples

```bash
# List all agents in table format
stencila agents list

# Output agents as JSON
stencila agents list --as json

# List only workspace agents
stencila agents list --source workspace

# List only CLI-detected agents
stencila agents list --source cli
```

# Options

| Name           | Description                                                               |
| -------------- | ------------------------------------------------------------------------- |
| `-a, --as`     | Output the list as JSON or YAML. Possible values: `json`, `yaml`, `toml`. |
| `-s, --source` | Filter by source (may be repeated).                                       |

**Possible values of `--source`**

| Value       | Description                                 |
| ----------- | ------------------------------------------- |
| `workspace` | `.stencila/agents/` in the workspace        |
| `user`      | `~/.config/stencila/agents/` (user-level)   |
| `builtin`   | Embedded in the binary                      |
| `cli`       | Auto-detected from a CLI tool found on PATH |
