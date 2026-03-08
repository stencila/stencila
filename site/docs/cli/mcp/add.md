---
title: "`stencila mcp add`"
description: Add an MCP server to stencila.toml
---

Add an MCP server to stencila.toml

Adds a new server configuration to the nearest `stencila.toml` file, or to the user-level config with `--user`.

The server spec is either a URL (for HTTP servers) or a command followed by its arguments (for stdio servers). URLs are detected by an `http://` or `https://` prefix.

# Usage

```sh
stencila mcp add [OPTIONS] <ID> <SPEC>...
```

# Examples

```bash
# Add a stdio server
stencila mcp add filesystem npx -y @modelcontextprotocol/server-filesystem /tmp

# Add an HTTP server
stencila mcp add remote-api https://api.example.com/mcp

# Add with a display name
stencila mcp add --name 'Filesystem Server' fs npx -y @modelcontextprotocol/server-filesystem /tmp

# Add with environment variables
stencila mcp add --env GITHUB_TOKEN=ghp_xxx github npx -y @modelcontextprotocol/server-github

# Add to user-level config
stencila mcp add --user my-server my-mcp-server

# Compatibility scope syntax
stencila mcp add --scope user my-server my-mcp-server
```

# Arguments

| Name     | Description                        |
| -------- | ---------------------------------- |
| `<ID>`   | The server ID (unique identifier). |
| `<SPEC>` | Command and arguments, or URL.     |

# Options

| Name          | Description                                                                              |
| ------------- | ---------------------------------------------------------------------------------------- |
| `--name`      | Human-readable name for the server.                                                      |
| `--env`       | Environment variable for stdio servers (repeatable, KEY=VALUE).                          |
| `-f, --force` | Overwrite if a server with this ID already exists. Possible values: `true`, `false`.     |
| `--user`      | Add to user config (~/.config/stencila/stencila.toml). Possible values: `true`, `false`. |
| `--workspace` | Add to workspace config (nearest stencila.toml). Possible values: `true`, `false`.       |
| `--scope`     | Config scope (compatibility with other tools). Possible values: `user`, `workspace`.     |
| `--dir`       | Workspace directory. Default value: `.`.                                                 |
