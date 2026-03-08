---
title: "`stencila mcp remove`"
description: Remove an MCP server from stencila.toml
---

Remove an MCP server from stencila.toml

Removes a server configuration from the nearest `stencila.toml` file, or from the user-level config with `--user`. Only Stencila-managed servers can be removed (not those from Claude, Codex, or Gemini configs).

# Usage

```sh
stencila mcp remove [OPTIONS] <ID>
```

# Examples

```bash
# Remove from nearest stencila.toml
stencila mcp remove filesystem

# Remove from user config
stencila mcp remove --user filesystem

# Compatibility scope syntax
stencila mcp remove --scope user filesystem
```

# Arguments

| Name   | Description              |
| ------ | ------------------------ |
| `<ID>` | The server ID to remove. |

# Options

| Name          | Description                                                                                   |
| ------------- | --------------------------------------------------------------------------------------------- |
| `--user`      | Remove from user config (~/.config/stencila/stencila.toml). Possible values: `true`, `false`. |
| `--workspace` | Remove from workspace config (nearest stencila.toml). Possible values: `true`, `false`.       |
| `--scope`     | Config scope (compatibility with other tools). Possible values: `user`, `workspace`.          |
