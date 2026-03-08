---
title: "`stencila mcp list`"
description: List all discovered MCP servers
---

List all discovered MCP servers

Shows servers from all sources: Stencila config, Claude, Codex, and Gemini. Servers are discovered from both user-level and workspace-level configs.

# Usage

```sh
stencila mcp list [OPTIONS]
```

# Examples

```bash
# List all servers in table format
stencila mcp list

# Output as JSON
stencila mcp list --as json

# List servers for a specific workspace
stencila mcp list --dir ./my-project
```

# Options

| Name       | Description                                                            |
| ---------- | ---------------------------------------------------------------------- |
| `-a, --as` | Output format (json or yaml). Possible values: `json`, `yaml`, `toml`. |
| `--dir`    | Workspace directory to discover servers from. Default value: `.`.      |
