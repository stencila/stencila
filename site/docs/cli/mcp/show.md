---
title: "`stencila mcp show`"
description: Show details of an MCP server
---

Show details of an MCP server

Displays the configuration for a server discovered from any source. Use `--tools` to connect to the server and list its available tools.

# Usage

```sh
stencila mcp show [OPTIONS] <ID>
```

# Examples

```bash
# Show server configuration
stencila mcp show filesystem

# Show as JSON
stencila mcp show filesystem --as json

# Connect and list tools
stencila mcp show filesystem --tools
```

# Arguments

| Name   | Description            |
| ------ | ---------------------- |
| `<ID>` | The server ID to show. |

# Options

| Name       | Description                                                            |
| ---------- | ---------------------------------------------------------------------- |
| `--tools`  | Connect and list the server's tools. Possible values: `true`, `false`. |
| `-a, --as` | Output format (json or yaml). Possible values: `json`, `yaml`, `toml`. |
| `--dir`    | Workspace directory. Default value: `.`.                               |
